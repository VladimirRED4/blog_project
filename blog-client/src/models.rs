use serde::{Deserialize, Serialize};

// ==================== Модели пользователей ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// ==================== Модели постов ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostsResponse {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ==================== Общие ошибки ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ==================== Вспомогательные функции ====================

impl From<crate::proto::User> for User {
    fn from(proto_user: crate::proto::User) -> Self {
        Self {
            id: proto_user.id,
            username: proto_user.username,
            email: proto_user.email,
            created_at: proto_user.created_at,
        }
    }
}

impl From<crate::proto::Post> for Post {
    fn from(proto_post: crate::proto::Post) -> Self {
        Self {
            id: proto_post.id,
            title: proto_post.title,
            content: proto_post.content,
            author_id: proto_post.author_id,
            created_at: proto_post.created_at,
            updated_at: proto_post.updated_at,
        }
    }
}