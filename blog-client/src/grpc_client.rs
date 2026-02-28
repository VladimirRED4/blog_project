use crate::error::BlogClientError;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

pub use crate::proto::{
    auth_service_client::AuthServiceClient, post_service_client::PostServiceClient,
    CreatePostRequest, DeletePostRequest, GetPostRequest, ListPostsRequest, ListPostsResponse,
    LoginRequest, LoginResponse, Post, RegisterRequest, RegisterResponse, UpdatePostRequest, User,
};

#[derive(Debug, Clone)]
pub struct GrpcClient {
    auth_client: AuthServiceClient<Channel>,
    post_client: PostServiceClient<Channel>,
    token: Option<String>,
}

impl GrpcClient {
    pub async fn new(addr: impl Into<String>) -> Result<Self, BlogClientError> {
        let addr = addr.into();
        let channel = Channel::from_shared(addr.clone())?.connect().await?;
        Ok(Self {
            auth_client: AuthServiceClient::new(channel.clone()),
            post_client: PostServiceClient::new(channel),
            token: None,
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    fn add_auth_header<T>(&self, mut request: Request<T>) -> Request<T> {
        if let Some(token) = &self.token {
            let auth_value = format!("Bearer {}", token)
                .parse::<MetadataValue<_>>()
                .expect("Failed to create auth header");
            request.metadata_mut().insert("authorization", auth_value);
        }
        request
    }

    // Auth methods
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<RegisterResponse, BlogClientError> {
        let request = Request::new(RegisterRequest {
            username,
            email,
            password,
        });

        let response = self.auth_client.clone().register(request).await?;

        let register_response = response.into_inner();

        if !register_response.token.is_empty() {
            self.set_token(register_response.token.clone());
        }

        Ok(register_response)
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<LoginResponse, BlogClientError> {
        let request = Request::new(LoginRequest {
            username,
            email: "".to_string(),
            password,
        });

        let response = self.auth_client.clone().login(request).await?;

        // Сохраняем токен, если он есть в ответе
        let token = response.get_ref().token.clone();
        if !token.is_empty() {
            self.set_token(token);
        }

        Ok(response.into_inner())
    }

    // Post methods
    pub async fn create_post(
        &self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let request = self.add_auth_header(Request::new(CreatePostRequest {
            title,
            content,
            author_id: 0,
            tags: vec![],
            published: true,
        }));

        let response = self.post_client.clone().create_post(request).await?;
        Ok(response.into_inner())
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let request = Request::new(GetPostRequest { id });
        let response = self.post_client.clone().get_post(request).await?;
        Ok(response.into_inner())
    }

    pub async fn update_post(
        &self,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, BlogClientError> {
        let request = self.add_auth_header(Request::new(UpdatePostRequest {
            id,
            title,
            content,
            tags: vec![],
            published: None,
        }));

        let response = self.post_client.clone().update_post(request).await?;
        Ok(response.into_inner())
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        let request = self.add_auth_header(Request::new(DeletePostRequest {
            id,
            token: "".to_string(),
        }));

        let response = self.post_client.clone().delete_post(request).await?;
        let result = response.into_inner();

        if result.success {
            Ok(())
        } else {
            Err(BlogClientError::TransportError(result.message))
        }
    }

    pub async fn list_posts(
        &self,
        page: i32,
        page_size: i32,
    ) -> Result<ListPostsResponse, BlogClientError> {
        let request = Request::new(ListPostsRequest {
            page,
            page_size,
            author_username: "".to_string(),
            tag: "".to_string(),
            published_only: true,
            search_query: "".to_string(),
        });

        let response = self.post_client.clone().list_posts(request).await?;
        Ok(response.into_inner())
    }
}
