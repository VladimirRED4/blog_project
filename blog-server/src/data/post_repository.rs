use crate::domain::post::{CreatePostRequest, UpdatePostRequest};
use crate::domain::{DomainError, Post};
use async_trait::async_trait;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, author_id: i64, req: CreatePostRequest) -> Result<Post, DomainError>;
    async fn find_by_id(&self, id: i64) -> Result<Post, DomainError>;
    async fn update(&self, id: i64, req: UpdatePostRequest) -> Result<Post, DomainError>;
    async fn delete(&self, id: i64) -> Result<(), DomainError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError>; // i64 для пагинации
    async fn find_by_author(&self, author_id: i64) -> Result<Vec<Post>, DomainError>;
}

pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, author_id: i64, req: CreatePostRequest) -> Result<Post, DomainError> {
        let row = sqlx::query(
            r#"
            INSERT INTO posts (title, content, author_id, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(&req.title)
        .bind(&req.content)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create post: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        let post = Post {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            content: row.try_get("content")?,
            author_id: row.try_get("author_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };

        Ok(post)
    }

    async fn find_by_id(&self, id: i64) -> Result<Post, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let post = Post {
                    id: row.try_get("id")?,
                    title: row.try_get("title")?,
                    content: row.try_get("content")?,
                    author_id: row.try_get("author_id")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                };
                Ok(post)
            }
            None => Err(DomainError::PostNotFound),
        }
    }

    async fn update(&self, id: i64, req: UpdatePostRequest) -> Result<Post, DomainError> {
        let row = sqlx::query(
            r#"
            UPDATE posts
            SET
                title = COALESCE($1, title),
                content = COALESCE($2, content),
                updated_at = NOW()
            WHERE id = $3
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(req.title)
        .bind(req.content)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let post = Post {
                    id: row.try_get("id")?,
                    title: row.try_get("title")?,
                    content: row.try_get("content")?,
                    author_id: row.try_get("author_id")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                };
                Ok(post)
            }
            None => Err(DomainError::PostNotFound),
        }
    }

    async fn delete(&self, id: i64) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            Err(DomainError::PostNotFound)
        } else {
            Ok(())
        }
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError> {
        // Get total count
        let count_row = sqlx::query("SELECT COUNT(*) as count FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.try_get("count")?;

        // Get paginated posts
        let rows = sqlx::query(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let posts = rows
            .into_iter()
            .map(|row| {
                Ok(Post {
                    id: row.try_get("id")?,
                    title: row.try_get("title")?,
                    content: row.try_get("content")?,
                    author_id: row.try_get("author_id")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect::<Result<Vec<Post>, DomainError>>()?;

        Ok((posts, total))
    }

    async fn find_by_author(&self, author_id: i64) -> Result<Vec<Post>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            WHERE author_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(author_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let posts = rows
            .into_iter()
            .map(|row| {
                Ok(Post {
                    id: row.try_get("id")?,
                    title: row.try_get("title")?,
                    content: row.try_get("content")?,
                    author_id: row.try_get("author_id")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect::<Result<Vec<Post>, DomainError>>()?;

        Ok(posts)
    }
}
