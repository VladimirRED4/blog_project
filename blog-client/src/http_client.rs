use crate::error::BlogClientError;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostsResponse {
    pub posts: Vec<PostResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: base_url.into(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    fn add_auth_header(&self, mut request: RequestBuilder) -> RequestBuilder {
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        request
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub async fn register(
        &mut self,
        req: RegisterRequest,
    ) -> Result<AuthResponse, BlogClientError> {
        let url = self.url("/api/auth/register");
        let response = self.client.post(&url).json(&req).send().await?;

        self.handle_auth_response(response).await
    }

    pub async fn login(&mut self, req: LoginRequest) -> Result<AuthResponse, BlogClientError> {
        let url = self.url("/api/auth/login");
        let response = self.client.post(&url).json(&req).send().await?;

        self.handle_auth_response(response).await
    }

    async fn handle_auth_response(
        &mut self,
        response: reqwest::Response,
    ) -> Result<AuthResponse, BlogClientError> {
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED => {
                let auth_response = response.json::<AuthResponse>().await?;
                self.set_token(auth_response.token.clone());
                Ok(auth_response)
            }
            StatusCode::UNAUTHORIZED => {
                let error_text = response.text().await?;
                Err(BlogClientError::Unauthorized(error_text))
            }
            StatusCode::NOT_FOUND => Err(BlogClientError::NotFound),
            StatusCode::CONFLICT => {
                let error_text = response.text().await?;
                Err(BlogClientError::InvalidRequest(error_text))
            }
            _ => {
                let error_text = response.text().await?;
                Err(BlogClientError::TransportError(format!(
                    "HTTP {}: {}",
                    status, error_text
                )))
            }
        }
    }

    pub async fn create_post(
        &self,
        title: String,
        content: String,
    ) -> Result<PostResponse, BlogClientError> {
        let url = self.url("/api/protected/posts");
        let request = CreatePostRequest { title, content };

        let response = self
            .add_auth_header(self.client.post(&url))
            .json(&request)
            .send()
            .await?;

        self.handle_post_response(response).await
    }

    pub async fn get_post(&self, id: i64) -> Result<PostResponse, BlogClientError> {
        let url = self.url(&format!("/api/posts/{}", id));
        let response = self.client.get(&url).send().await?;
        self.handle_post_response(response).await
    }

    pub async fn update_post(
        &self,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<PostResponse, BlogClientError> {
        let url = self.url(&format!("/api/protected/posts/{}", id));
        let request = UpdatePostRequest { title, content };

        let response = self
            .add_auth_header(self.client.put(&url))
            .json(&request)
            .send()
            .await?;

        self.handle_post_response(response).await
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        let url = self.url(&format!("/api/protected/posts/{}", id));
        let response = self
            .add_auth_header(self.client.delete(&url))
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::UNAUTHORIZED => {
                let error_text = response.text().await?;
                Err(BlogClientError::Unauthorized(error_text))
            }
            StatusCode::NOT_FOUND => Err(BlogClientError::NotFound),
            _ => {
                let error_text = response.text().await?;
                Err(BlogClientError::TransportError(format!(
                    "HTTP {}: {}",
                    status, error_text
                )))
            }
        }
    }

    pub async fn list_posts(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<PostsResponse, BlogClientError> {
        let mut url = self.url("/api/posts");
        let mut params = vec![];

        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        match status {
            StatusCode::OK => {
                let posts_response = response.json::<PostsResponse>().await?;
                Ok(posts_response)
            }
            _ => {
                let error_text = response.text().await?;
                Err(BlogClientError::TransportError(format!(
                    "HTTP {}: {}",
                    status, error_text
                )))
            }
        }
    }

    async fn handle_post_response(
        &self,
        response: reqwest::Response,
    ) -> Result<PostResponse, BlogClientError> {
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED => {
                let post = response.json::<PostResponse>().await?;
                Ok(post)
            }
            StatusCode::UNAUTHORIZED => {
                let error_text = response.text().await?;
                Err(BlogClientError::Unauthorized(error_text))
            }
            StatusCode::NOT_FOUND => Err(BlogClientError::NotFound),
            StatusCode::FORBIDDEN => {
                let error_text = response.text().await?;
                Err(BlogClientError::InvalidRequest(format!(
                    "Forbidden: {}",
                    error_text
                )))
            }
            _ => {
                let error_text = response.text().await?;
                Err(BlogClientError::TransportError(format!(
                    "HTTP {}: {}",
                    status, error_text
                )))
            }
        }
    }
}
