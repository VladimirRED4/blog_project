use dotenvy::dotenv;
use std::sync::Arc;

mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

pub mod proto {
    tonic::include_proto!("blog");
}

use application::{auth_service::AuthService, blog_service::BlogService};
use data::{post_repository::PostgresPostRepository, user_repository::PostgresUserRepository};
use infrastructure::{
    database::{create_pool, run_migrations},
    jwt::JwtService,
    logging::init_logging,
};
use presentation::{grpc_service::BlogGrpcService, http_handlers, middleware::jwt_middleware};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    init_logging();

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let http_port = std::env::var("HTTP_PORT").unwrap_or_else(|_| "3000".to_string());
    let grpc_port = std::env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());

    // Получаем разрешенные CORS домены из .env
    let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8000,http://127.0.0.1:8000".to_string());

    let http_addr = format!("0.0.0.0:{}", http_port);
    let grpc_addr = format!("0.0.0.0:{}", grpc_port);

    tracing::info!("Starting blog server...");
    tracing::info!("HTTP server will listen on {}", http_addr);
    tracing::info!("gRPC server will listen on {}", grpc_addr);
    tracing::info!("CORS allowed origins: {}", cors_allowed_origins);

    // Initialize database connection pool
    tracing::info!("Connecting to database...");
    let pool = create_pool(&database_url).await?;

    // Run database migrations
    tracing::info!("Running database migrations...");
    run_migrations(&pool).await?;
    tracing::info!("Migrations completed successfully");

    // Initialize services
    tracing::info!("Initializing services...");

    // JWT service
    let jwt_service = Arc::new(JwtService::new(&jwt_secret)?);

    // Repositories
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let post_repo = Arc::new(PostgresPostRepository::new(pool.clone()));

    // Application services
    let auth_service = Arc::new(AuthService::new(user_repo.clone(), jwt_service.clone()));

    let blog_service = Arc::new(BlogService::new(post_repo.clone()));

    tracing::info!("Services initialized successfully");

    // Clone services for HTTP and gRPC servers
    let auth_service_http = auth_service.clone();
    let blog_service_http = blog_service.clone();
    let jwt_service_http = jwt_service.clone();

    let auth_service_grpc = auth_service.clone();
    let blog_service_grpc = blog_service.clone();
    let jwt_service_grpc = jwt_service.clone();

    // Start HTTP server (actix-web)
    tracing::info!("Starting HTTP server...");
    let http_server = tokio::spawn(async move {
        if let Err(e) = run_http_server(
            http_addr,
            auth_service_http,
            blog_service_http,
            jwt_service_http,
            cors_allowed_origins,
        )
        .await
        {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    // Start gRPC server (tonic)
    tracing::info!("Starting gRPC server...");
    let grpc_server = tokio::spawn(async move {
        if let Err(e) = run_grpc_server(
            grpc_addr,
            auth_service_grpc,
            blog_service_grpc,
            jwt_service_grpc,
        )
        .await
        {
            tracing::error!("gRPC server error: {}", e);
        }
    });

    // Wait for both servers to complete (they shouldn't, unless there's an error)
    tokio::select! {
        result = http_server => {
            match result {
                Ok(_) => tracing::info!("HTTP server stopped"),
                Err(e) => tracing::error!("HTTP server task failed: {}", e),
            }
        }
        result = grpc_server => {
            match result {
                Ok(_) => tracing::info!("gRPC server stopped"),
                Err(e) => tracing::error!("gRPC server task failed: {}", e),
            }
        }
    }

    tracing::info!("Shutting down...");
    Ok(())
}

/// Configure CORS for the HTTP server with allowed origins from .env
fn configure_cors(allowed_origins: &str) -> actix_cors::Cors {
    use actix_cors::Cors;
    use actix_web::http::header;

    tracing::info!("Configuring CORS with allowed origins: {}", allowed_origins);

    let origins: Vec<&str> = allowed_origins.split(',').map(|s| s.trim()).collect();

    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
        ])
        .expose_headers(vec![header::AUTHORIZATION])
        .max_age(3600);

    // Добавляем каждый разрешенный домен
    for origin in origins {
        if !origin.is_empty() {
            cors = cors.allowed_origin(origin);
            tracing::debug!("Added allowed CORS origin: {}", origin);
        }
    }

    cors
}

async fn run_http_server(
    addr: String,
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
    cors_allowed_origins: String,
) -> anyhow::Result<()> {
    use actix_web::{middleware::Logger, web, App, HttpServer};
    use actix_web_httpauth::middleware::HttpAuthentication;

    tracing::info!("Configuring HTTP server...");

    let auth_middleware = HttpAuthentication::bearer(jwt_middleware);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(configure_cors(&cors_allowed_origins))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(blog_service.clone()))
            .app_data(web::Data::new(jwt_service.clone()))
            // Public routes - authentication
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(http_handlers::register))
                    .route("/login", web::post().to(http_handlers::login)),
            )
            // Public routes - posts (read-only)
            .service(
                web::scope("/api/posts")
                    .route("", web::get().to(http_handlers::list_posts))
                    .route("/{id}", web::get().to(http_handlers::get_post)),
            )
            // Protected routes - posts (write operations)
            .service(
                web::scope("/api/protected/posts")
                    .wrap(auth_middleware.clone())
                    .route("", web::post().to(http_handlers::create_post))
                    .route("/{id}", web::put().to(http_handlers::update_post))
                    .route("/{id}", web::delete().to(http_handlers::delete_post)),
            )
    })
    .bind(&addr)?
    .run();

    tracing::info!("HTTP server running on {}", addr);

    server.await?;

    Ok(())
}

async fn run_grpc_server(
    addr: String,
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
) -> anyhow::Result<()> {
    use tonic::transport::Server;

    let grpc_service = BlogGrpcService::new(auth_service, blog_service, jwt_service);

    let addr = addr.parse()?;

    tracing::info!("gRPC server running on {}", addr);

    Server::builder()
        .add_service(crate::proto::auth_service_server::AuthServiceServer::new(
            grpc_service.clone(),
        ))
        .add_service(crate::proto::post_service_server::PostServiceServer::new(
            grpc_service,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
