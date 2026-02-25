use crate::domain::user::RegisterUserRequest;
use crate::domain::{DomainError, User};
use async_trait::async_trait;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        req: RegisterUserRequest,
        password_hash: String,
    ) -> Result<User, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<User, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<User, DomainError>;
    #[allow(dead_code)]
    async fn find_by_id(&self, id: i64) -> Result<User, DomainError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(
        &self,
        req: RegisterUserRequest,
        password_hash: String,
    ) -> Result<User, DomainError> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (username, email, password_hash, created_at)
            VALUES ($1, $2, $3, NOW())
            RETURNING id, username, email, password_hash, created_at
            "#,
        )
        .bind(&req.username)
        .bind(&req.email)
        .bind(&password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user: {}", e);
            if e.to_string().contains("duplicate key") {
                DomainError::UserAlreadyExists
            } else {
                DomainError::DatabaseError(e.to_string())
            }
        })?;

        let user = User {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            created_at: row.try_get("created_at")?,
        };

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<User, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.try_get("id")?,
                    username: row.try_get("username")?,
                    email: row.try_get("email")?,
                    password_hash: row.try_get("password_hash")?,
                    created_at: row.try_get("created_at")?,
                };
                Ok(user)
            }
            None => Err(DomainError::UserNotFound),
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<User, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.try_get("id")?,
                    username: row.try_get("username")?,
                    email: row.try_get("email")?,
                    password_hash: row.try_get("password_hash")?,
                    created_at: row.try_get("created_at")?,
                };
                Ok(user)
            }
            None => Err(DomainError::UserNotFound),
        }
    }

    async fn find_by_id(&self, id: i64) -> Result<User, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.try_get("id")?,
                    username: row.try_get("username")?,
                    email: row.try_get("email")?,
                    password_hash: row.try_get("password_hash")?,
                    created_at: row.try_get("created_at")?,
                };
                Ok(user)
            }
            None => Err(DomainError::UserNotFound),
        }
    }
}
