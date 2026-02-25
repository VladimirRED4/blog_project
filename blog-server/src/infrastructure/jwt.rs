use crate::domain::DomainError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    #[allow(dead_code)]
    secret_length: usize,
}

impl JwtService {
    pub fn new(secret: &str) -> Result<Self, DomainError> {
        tracing::debug!(
            "Initializing JwtService with secret length: {}",
            secret.len()
        );

        if secret.len() < 32 {
            tracing::warn!(
                "JWT secret is too short ({} chars). Minimum recommended is 32 chars.",
                secret.len()
            );
        }

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            secret_length: secret.len(),
        })
    }

    pub fn generate_token(&self, user_id: i64, username: String) -> Result<String, DomainError> {
        tracing::debug!(
            "Generating token for user_id: {}, username: {}",
            user_id,
            username
        );

        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            user_id,
            username,
            exp: expiration,
        };

        match encode(&Header::default(), &claims, &self.encoding_key) {
            Ok(token) => {
                tracing::debug!("Token encoded successfully");
                Ok(token)
            }
            Err(e) => {
                tracing::error!("Failed to encode token: {}", e);
                Err(DomainError::InternalError(format!(
                    "Failed to generate token: {}",
                    e
                )))
            }
        }
    }

    pub fn verify_token(&self, token: &str) -> Result<i64, DomainError> {
        match decode::<Claims>(token, &self.decoding_key, &Validation::default()) {
            Ok(token_data) => {
                tracing::debug!("Token verified for user_id: {}", token_data.claims.user_id);
                Ok(token_data.claims.user_id)
            }
            Err(e) => {
                tracing::error!("Token verification failed: {}", e);
                Err(DomainError::Unauthorized(format!("Invalid token: {}", e)))
            }
        }
    }
}
