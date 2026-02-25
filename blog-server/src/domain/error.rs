use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Post not found")]
    PostNotFound,

    #[error("Forbidden: you don't have permission to perform this action")]
    Forbidden,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl DomainError {
    pub fn to_status_code(&self) -> u16 {
        match self {
            Self::UserNotFound | Self::PostNotFound => 404,
            Self::UserAlreadyExists => 409,
            Self::InvalidCredentials | Self::Unauthorized(_) => 401,
            Self::Forbidden => 403,
            Self::ValidationError(_) => 400,
            Self::DatabaseError(_) | Self::InternalError(_) => 500,
        }
    }
}

impl From<sqlx::Error> for DomainError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::UserNotFound,
            _ => Self::DatabaseError(err.to_string()),
        }
    }
}
