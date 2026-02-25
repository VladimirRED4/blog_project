use thiserror::Error;
use tonic::transport::Error as GrpcTransportError;

#[derive(Debug, Error)]
pub enum BlogClientError {
    // HTTP ошибки
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    // gRPC ошибки
    #[error("gRPC error: {0}")]
    GrpcError(tonic::Status),

    #[error("gRPC transport error: {0}")]
    GrpcTransportError(#[from] GrpcTransportError),

    // Ошибки URI
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    // Бизнес-логика ошибки
    #[error("Resource not found")]
    NotFound,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    // Транспортные ошибки
    #[error("Transport error: {0}")]
    TransportError(String),

    // Ошибки сериализации/десериализации
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl BlogClientError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, BlogClientError::NotFound)
    }

    pub fn is_unauthorized(&self) -> bool {
        matches!(self, BlogClientError::Unauthorized(_))
    }
}

// Реализация From для tonic::Status
impl From<tonic::Status> for BlogClientError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => BlogClientError::NotFound,
            tonic::Code::Unauthenticated => {
                BlogClientError::Unauthorized(status.message().to_string())
            }
            tonic::Code::InvalidArgument => {
                BlogClientError::InvalidRequest(status.message().to_string())
            }
            _ => BlogClientError::GrpcError(status),
        }
    }
}
