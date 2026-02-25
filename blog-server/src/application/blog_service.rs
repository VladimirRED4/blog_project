use crate::data::post_repository::PostRepository;
use crate::domain::post::{CreatePostRequest, PostResponse, UpdatePostRequest};
use crate::domain::DomainError;
use std::sync::Arc;

pub struct BlogService {
    post_repo: Arc<dyn PostRepository + Send + Sync>,
}

impl BlogService {
    pub fn new(post_repo: Arc<dyn PostRepository + Send + Sync>) -> Self {
        Self { post_repo }
    }

    pub async fn create_post(
        &self,
        author_id: i64,
        req: CreatePostRequest,
    ) -> Result<PostResponse, DomainError> {
        // Validate input
        if req.title.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Title cannot be empty".to_string(),
            ));
        }
        if req.content.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Content cannot be empty".to_string(),
            ));
        }

        // Create post
        let post = self.post_repo.create(author_id, req).await?;

        tracing::info!("Post created: id={}, author_id={}", post.id, author_id);

        Ok(PostResponse::from(post))
    }

    pub async fn get_post(&self, id: i64) -> Result<PostResponse, DomainError> {
        let post = self.post_repo.find_by_id(id).await?;
        Ok(PostResponse::from(post))
    }

    pub async fn update_post(
        &self,
        id: i64,
        user_id: i64,
        req: UpdatePostRequest,
    ) -> Result<PostResponse, DomainError> {
        // Check if post exists and user is author
        let post = self.post_repo.find_by_id(id).await?;

        if post.author_id != user_id {
            tracing::warn!(
                "User {} attempted to update post {} owned by {}",
                user_id,
                id,
                post.author_id
            );
            return Err(DomainError::Forbidden);
        }

        // Update post
        let updated_post = self.post_repo.update(id, req).await?;

        tracing::info!("Post updated: id={}, author_id={}", id, user_id);

        Ok(PostResponse::from(updated_post))
    }

    pub async fn delete_post(&self, id: i64, user_id: i64) -> Result<(), DomainError> {
        // Check if post exists and user is author
        let post = self.post_repo.find_by_id(id).await?;

        if post.author_id != user_id {
            tracing::warn!(
                "User {} attempted to delete post {} owned by {}",
                user_id,
                id,
                post.author_id
            );
            return Err(DomainError::Forbidden);
        }

        // Delete post
        self.post_repo.delete(id).await?;

        tracing::info!("Post deleted: id={}, author_id={}", id, user_id);

        Ok(())
    }

    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PostResponse>, i64), DomainError> {
        // Validate pagination parameters
        if !(1..=100).contains(&limit) {
            return Err(DomainError::ValidationError(
                "Limit must be between 1 and 100".to_string(),
            ));
        }
        if offset < 0 {
            return Err(DomainError::ValidationError(
                "Offset cannot be negative".to_string(),
            ));
        }

        let (posts, total) = self.post_repo.list(limit, offset).await?;

        let post_responses = posts.into_iter().map(PostResponse::from).collect();

        Ok((post_responses, total))
    }

    #[allow(dead_code)]
    pub async fn get_user_posts(&self, author_id: i64) -> Result<Vec<PostResponse>, DomainError> {
        let posts = self.post_repo.find_by_author(author_id).await?;

        Ok(posts.into_iter().map(PostResponse::from).collect())
    }
}
