use crate::application::{AuthService, BlogService};
use crate::domain::post::{CreatePostRequest, PostResponse, UpdatePostRequest};
use crate::domain::user::{LoginUserRequest, RegisterUserRequest, UserResponse};
use crate::domain::DomainError;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

// Структура для ответа с токеном
#[derive(serde::Serialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

// Структура для пагинации
#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Структура для ответа со списком постов
#[derive(serde::Serialize)]
struct PostsResponse {
    posts: Vec<PostResponse>,
    total: i64,
    limit: i64,
    offset: i64,
}

// Вспомогательная функция для извлечения user_id из request extensions
fn get_user_id_from_request(req: &HttpRequest) -> Result<i64, DomainError> {
    req.extensions()
        .get::<i64>()
        .copied()
        .ok_or(DomainError::Unauthorized(
            "User not authenticated".to_string(),
        ))
}

// Преобразование DomainError в HttpResponse
fn error_to_response(err: DomainError) -> HttpResponse {
    let status_code = err.to_status_code();
    let message = err.to_string();

    match status_code {
        400 => HttpResponse::BadRequest().json(serde_json::json!({ "error": message })),
        401 => HttpResponse::Unauthorized().json(serde_json::json!({ "error": message })),
        403 => HttpResponse::Forbidden().json(serde_json::json!({ "error": message })),
        404 => HttpResponse::NotFound().json(serde_json::json!({ "error": message })),
        409 => HttpResponse::Conflict().json(serde_json::json!({ "error": message })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "Internal server error" })),
    }
}

// ============== Auth Handlers ==============

pub async fn register(
    auth_service: web::Data<Arc<AuthService>>,
    req: web::Json<RegisterUserRequest>,
) -> impl Responder {
    match auth_service.register(req.into_inner()).await {
        Ok((token, user)) => HttpResponse::Created().json(AuthResponse { token, user }),
        Err(err) => error_to_response(err),
    }
}

pub async fn login(
    auth_service: web::Data<Arc<AuthService>>,
    req: web::Json<LoginUserRequest>,
) -> impl Responder {
    match auth_service.login(req.into_inner()).await {
        Ok((token, user)) => HttpResponse::Ok().json(AuthResponse { token, user }),
        Err(err) => error_to_response(err),
    }
}

// ============== Post Handlers ==============

pub async fn list_posts(
    blog_service: web::Data<Arc<BlogService>>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    tracing::info!("Listing posts with limit={}, offset={}", limit, offset);

    match blog_service.list_posts(limit, offset).await {
        Ok((posts, total)) => HttpResponse::Ok().json(PostsResponse {
            posts,
            total,
            limit,
            offset,
        }),
        Err(err) => error_to_response(err),
    }
}

pub async fn get_post(
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> impl Responder {
    let post_id = path.into_inner();

    tracing::info!("Getting post with id={}", post_id);

    match blog_service.get_post(post_id).await {
        // post_id уже i64
        Ok(post) => HttpResponse::Ok().json(post),
        Err(err) => error_to_response(err),
    }
}

pub async fn create_post(
    req: HttpRequest,
    blog_service: web::Data<Arc<BlogService>>,
    post_data: web::Json<CreatePostRequest>,
) -> impl Responder {
    // Extract user_id from JWT middleware
    let user_id = match get_user_id_from_request(&req) {
        Ok(id) => id,
        Err(err) => return error_to_response(err),
    };

    tracing::info!("Creating post for user_id={}", user_id);

    match blog_service
        .create_post(user_id, post_data.into_inner())
        .await
    {
        Ok(post) => HttpResponse::Created().json(post),
        Err(err) => error_to_response(err),
    }
}

pub async fn update_post(
    req: HttpRequest,
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
    post_data: web::Json<UpdatePostRequest>,
) -> impl Responder {
    let post_id = path.into_inner();

    // Extract user_id from JWT middleware
    let user_id = match get_user_id_from_request(&req) {
        Ok(id) => id,
        Err(err) => return error_to_response(err),
    };

    tracing::info!("Updating post id={} for user_id={}", post_id, user_id);

    match blog_service
        .update_post(post_id, user_id, post_data.into_inner())
        .await
    {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(err) => error_to_response(err),
    }
}

pub async fn delete_post(
    req: HttpRequest,
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> impl Responder {
    let post_id = path.into_inner();

    // Extract user_id from JWT middleware
    let user_id = match get_user_id_from_request(&req) {
        Ok(id) => id,
        Err(err) => return error_to_response(err),
    };

    tracing::info!("Deleting post id={} for user_id={}", post_id, user_id);

    match blog_service.delete_post(post_id, user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => error_to_response(err),
    }
}
