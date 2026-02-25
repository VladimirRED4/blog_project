use crate::application::{AuthService, BlogService};
use crate::domain::post::{
    CreatePostRequest as DomainCreatePostRequest, UpdatePostRequest as DomainUpdatePostRequest,
};
use crate::domain::user::{
    LoginUserRequest as DomainLoginRequest, RegisterUserRequest as DomainRegisterRequest,
};
use crate::infrastructure::jwt::JwtService;
use crate::proto::*;
use std::sync::Arc;
use tonic::{Request, Response, Status};

// Вспомогательная функция для извлечения user_id из JWT
#[allow(clippy::result_large_err)]
fn extract_user_id_from_token(token: &str, jwt_service: &JwtService) -> Result<i64, Status> {
    // Remove "Bearer " prefix if present
    let token = token.strip_prefix("Bearer ").unwrap_or(token);

    jwt_service
        .verify_token(token)
        .map_err(|_| Status::unauthenticated("Invalid or expired token"))
}

// Преобразование доменных ошибок в gRPC статусы
fn map_domain_error(err: crate::domain::DomainError) -> Status {
    match err {
        crate::domain::DomainError::UserNotFound => Status::not_found("User not found"),
        crate::domain::DomainError::PostNotFound => Status::not_found("Post not found"),
        crate::domain::DomainError::UserAlreadyExists => {
            Status::already_exists("User already exists")
        }
        crate::domain::DomainError::InvalidCredentials => {
            Status::unauthenticated("Invalid credentials")
        }
        crate::domain::DomainError::Forbidden => Status::permission_denied("Forbidden"),
        crate::domain::DomainError::ValidationError(msg) => Status::invalid_argument(msg),
        crate::domain::DomainError::Unauthorized(msg) => Status::unauthenticated(msg),
        crate::domain::DomainError::DatabaseError(msg) => {
            Status::internal(format!("Database error: {}", msg))
        }
        crate::domain::DomainError::InternalError(msg) => Status::internal(msg),
    }
}

// Преобразование доменного User в protobuf User
fn user_to_proto(user: crate::domain::user::UserResponse) -> User {
    User {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: "".to_string(),
        bio: "".to_string(),
        avatar_url: "".to_string(),
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.created_at.to_rfc3339(),
    }
}

// Преобразование доменного Post в protobuf Post
fn post_to_proto(post: crate::domain::post::PostResponse) -> Post {
    Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        author: None,
        tags: vec![],
        likes_count: 0,
        views_count: 0,
        created_at: post.created_at.to_rfc3339(),
        updated_at: post.updated_at.to_rfc3339(),
        published: true,
        published_at: post.created_at.to_rfc3339(),
    }
}

#[derive(Clone)]
pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<BlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }
}

#[tonic::async_trait]
impl auth_service_server::AuthService for BlogGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        let register_req = DomainRegisterRequest {
            username: req.username,
            email: req.email,
            password: req.password,
            // full_name: req.full_name,
        };

        match self.auth_service.register(register_req).await {
            Ok((_token, user)) => {
                let response = RegisterResponse {
                    user_id: user.id,
                    message: "User registered successfully".to_string(),
                };
                Ok(Response::new(response))
            }
            Err(err) => Err(map_domain_error(err)),
        }
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let login_req = if !req.username.is_empty() {
            DomainLoginRequest {
                username: req.username,
                password: req.password,
            }
        } else {
            DomainLoginRequest {
                username: req.email,
                password: req.password,
            }
        };

        match self.auth_service.login(login_req).await {
            Ok((token, user)) => {
                let response = LoginResponse {
                    token,
                    refresh_token: "".to_string(),
                    user: Some(user_to_proto(user)),
                    expires_in: 86400,
                };
                Ok(Response::new(response))
            }
            Err(err) => Err(map_domain_error(err)),
        }
    }

    async fn logout(
        &self,
        _request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        Ok(Response::new(LogoutResponse {
            success: true,
            message: "Logged out successfully".to_string(),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        match self.jwt_service.verify_token(&req.token) {
            Ok(user_id) => {
                let response = ValidateTokenResponse {
                    valid: true,
                    user_id,
                    user: None,
                };
                Ok(Response::new(response))
            }
            Err(_) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                user_id: 0,
                user: None,
            })),
        }
    }
}

#[tonic::async_trait]
impl post_service_server::PostService for BlogGrpcService {
    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<Post>, Status> {
        // Извлекаем токен из метаданных
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing authorization token"))?;

        let user_id = extract_user_id_from_token(token, &self.jwt_service)?;

        let req = request.into_inner();

        // Создаем доменный запрос из protobuf
        let create_req = DomainCreatePostRequest {
            title: req.title,
            content: req.content,
        };

        match self.blog_service.create_post(user_id, create_req).await {
            Ok(post) => Ok(Response::new(post_to_proto(post))),
            Err(err) => Err(map_domain_error(err)),
        }
    }

    async fn get_post(&self, request: Request<GetPostRequest>) -> Result<Response<Post>, Status> {
        let req = request.into_inner();

        match self.blog_service.get_post(req.id).await {
            Ok(post) => Ok(Response::new(post_to_proto(post))),
            Err(err) => Err(map_domain_error(err)),
        }
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<Post>, Status> {
        // Извлекаем токен из метаданных
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing authorization token"))?;

        let user_id = extract_user_id_from_token(token, &self.jwt_service)?;

        let req = request.into_inner();

        // Создаем доменный запрос из protobuf
        // В protobuf UpdatePostRequest поля опциональные
        let update_req = DomainUpdatePostRequest {
            title: req.title,
            content: req.content,
        };

        match self
            .blog_service
            .update_post(req.id, user_id, update_req)
            .await
        {
            Ok(post) => Ok(Response::new(post_to_proto(post))),
            Err(err) => Err(map_domain_error(err)),
        }
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        // Извлекаем токен из метаданных
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing authorization token"))?;

        let user_id = extract_user_id_from_token(token, &self.jwt_service)?;

        let req = request.into_inner();

        match self.blog_service.delete_post(req.id, user_id).await {
            Ok(()) => Ok(Response::new(DeletePostResponse {
                success: true,
                message: format!("Post {} deleted", req.id),
            })),
            Err(err) => Err(map_domain_error(err)),
        }
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();

        let limit = if req.page_size > 0 && req.page_size <= 100 {
            req.page_size as i64
        } else {
            10
        };

        let offset = if req.page > 0 {
            ((req.page - 1) * limit as i32) as i64
        } else {
            0
        };

        match self.blog_service.list_posts(limit, offset).await {
            Ok((posts, total)) => {
                let response = ListPostsResponse {
                    posts: posts.into_iter().map(post_to_proto).collect(),
                    total_count: total as i32,
                    page: req.page,
                    page_size: req.page_size,
                    total_pages: ((total + limit - 1) / limit) as i32,
                };
                Ok(Response::new(response))
            }
            Err(err) => Err(map_domain_error(err)),
        }
    }
}
