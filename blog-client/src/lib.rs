pub mod error;
pub mod grpc_client;
pub mod http_client;

pub mod proto {
    tonic::include_proto!("blog");
}

use error::BlogClientError;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Transport type for the client
#[derive(Debug, Clone, PartialEq)]
pub enum Transport {
    /// HTTP transport with base URL (e.g., "http://localhost:3000")
    Http(String),
    /// gRPC transport with server address (e.g., "http://localhost:50051")
    Grpc(String),
}

/// Unified Blog Client that can use either HTTP or gRPC transport
#[derive(Debug, Clone)]
pub struct BlogClient {
    transport: Transport,
    http_client: Option<Arc<Mutex<http_client::HttpClient>>>,
    grpc_client: Option<Arc<Mutex<grpc_client::GrpcClient>>>,
    token: Arc<Mutex<Option<String>>>,
}

impl BlogClient {
    /// Create a new blog client with the specified transport
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        match &transport {
            Transport::Http(base_url) => {
                let http_client = http_client::HttpClient::new(base_url.clone());
                Ok(Self {
                    transport,
                    http_client: Some(Arc::new(Mutex::new(http_client))),
                    grpc_client: None,
                    token: Arc::new(Mutex::new(None)),
                })
            }
            Transport::Grpc(addr) => {
                let grpc_client = grpc_client::GrpcClient::new(addr.clone()).await?;
                Ok(Self {
                    transport,
                    http_client: None,
                    grpc_client: Some(Arc::new(Mutex::new(grpc_client))),
                    token: Arc::new(Mutex::new(None)),
                })
            }
        }
    }

    /// Set the JWT token for authenticated requests
    pub async fn set_token(&self, token: String) {
        let mut token_lock = self.token.lock().await;
        *token_lock = Some(token.clone());

        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    let mut http = client.lock().await;
                    http.set_token(token);
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &self.grpc_client {
                    let mut grpc = client.lock().await;
                    grpc.set_token(token);
                }
            }
        }
    }

    /// Get the current JWT token
    pub async fn get_token(&self) -> Option<String> {
        self.token.lock().await.clone()
    }

    /// Clear the current JWT token (logout)
    pub async fn clear_token(&self) {
        let mut token_lock = self.token.lock().await;
        *token_lock = None;
    }

    /// Register a new user
    pub async fn register(
        &self,
        username: impl Into<String>,
        email: impl Into<String>,
        password: impl Into<String>,
        full_name: impl Into<String>,
    ) -> Result<http_client::AuthResponse, BlogClientError> {
        let username = username.into();
        let email = email.into();
        let password = password.into();
        let full_name = full_name.into();

        tracing::debug!("Register called for username: {}", username);

        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    let mut http = client.lock().await;
                    tracing::debug!("Got HTTP client lock");

                    let req = http_client::RegisterRequest {
                        username: username.clone(),
                        email: email.clone(),
                        password,
                        full_name,
                    };

                    tracing::debug!("Sending register request...");
                    let response = http.register(req).await?;
                    tracing::debug!("Register response received, setting token...");

                    if let Some(token) = http.get_token() {
                        tracing::debug!("Setting token in main client");
                        let token = token.clone();
                        let token_clone = self.token.clone();
                        tokio::spawn(async move {
                            let mut token_lock = token_clone.lock().await;
                            *token_lock = Some(token);
                        });
                    }

                    tracing::debug!("Returning response");
                    Ok(response)
                } else {
                    Err(BlogClientError::TransportError(
                        "HTTP client not initialized".into(),
                    ))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &self.grpc_client {
                    let mut grpc = client.lock().await;
                    tracing::debug!("Got gRPC client lock for register");

                    let response = grpc
                        .register(username.clone(), email.clone(), password, full_name)
                        .await?;
                    tracing::debug!(
                        "gRPC register response received, user_id: {}",
                        response.user_id
                    );

                    Ok(http_client::AuthResponse {
                        token: "".to_string(),
                        user: http_client::UserResponse {
                            id: response.user_id,
                            username,
                            email,
                            created_at: chrono::Utc::now().to_rfc3339(),
                        },
                    })
                } else {
                    Err(BlogClientError::TransportError(
                        "gRPC client not initialized".into(),
                    ))
                }
            }
        }
    }

    /// Login with username and password
    pub async fn login(
        &self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<http_client::AuthResponse, BlogClientError> {
        let username = username.into();
        let password = password.into();

        tracing::debug!("Login called for username: {}", username);

        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    let mut http = client.lock().await;
                    tracing::debug!("Got HTTP client lock");

                    let req = http_client::LoginRequest {
                        username: username.clone(),
                        password,
                    };

                    tracing::debug!("Sending login request...");
                    let response = http.login(req).await?;
                    tracing::debug!("Login response received, setting token...");

                    if let Some(token) = http.get_token() {
                        tracing::debug!("Setting token in main client");
                        let token = token.clone();
                        let token_clone = self.token.clone();
                        tokio::spawn(async move {
                            let mut token_lock = token_clone.lock().await;
                            *token_lock = Some(token);
                        });
                    }

                    tracing::debug!("Returning response");
                    Ok(response)
                } else {
                    Err(BlogClientError::TransportError(
                        "HTTP client not initialized".into(),
                    ))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &self.grpc_client {
                    let mut grpc = client.lock().await;
                    tracing::debug!("Got gRPC client lock for login");

                    let response = grpc.login(username.clone(), password).await?;
                    tracing::debug!("gRPC login response received, token received");

                    if !response.token.is_empty() {
                        let token = response.token.clone();
                        let token_clone = self.token.clone();
                        tokio::spawn(async move {
                            let mut token_lock = token_clone.lock().await;
                            *token_lock = Some(token);
                        });
                    }

                    if let Some(user) = response.user {
                        Ok(http_client::AuthResponse {
                            token: response.token,
                            user: http_client::UserResponse {
                                id: user.id,
                                username,
                                email: user.email,
                                created_at: user.created_at,
                            },
                        })
                    } else {
                        Err(BlogClientError::InvalidRequest(
                            "No user data in response".into(),
                        ))
                    }
                } else {
                    Err(BlogClientError::TransportError(
                        "gRPC client not initialized".into(),
                    ))
                }
            }
        }
    }

    /// Create a new post (requires authentication)
    pub async fn create_post(
        &self,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<http_client::PostResponse, BlogClientError> {
        let title = title.into();
        let content = content.into();

        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    let http = client.lock().await;
                    http.create_post(title, content).await
                } else {
                    Err(BlogClientError::TransportError(
                        "HTTP client not initialized".into(),
                    ))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &self.grpc_client {
                    let grpc = client.lock().await;
                    let post = grpc.create_post(title, content).await?;

                    Ok(http_client::PostResponse {
                        id: post.id,
                        title: post.title,
                        content: post.content,
                        author_id: post.author_id,
                        created_at: post.created_at,
                        updated_at: post.updated_at,
                    })
                } else {
                    Err(BlogClientError::TransportError(
                        "gRPC client not initialized".into(),
                    ))
                }
            }
        }
    }

    /// Get a post by ID
    pub async fn get_post(&self, id: i64) -> Result<http_client::PostResponse, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    let http = client.lock().await;
                    http.get_post(id).await
                } else {
                    Err(BlogClientError::TransportError(
                        "HTTP client not initialized".into(),
                    ))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &self.grpc_client {
                    let grpc = client.lock().await;
                    let post = grpc.get_post(id).await?;

                    Ok(http_client::PostResponse {
                        id: post.id,
                        title: post.title,
                        content: post.content,
                        author_id: post.author_id,
                        created_at: post.created_at,
                        updated_at: post.updated_at,
                    })
                } else {
                    Err(BlogClientError::TransportError(
                        "gRPC client not initialized".into(),
                    ))
                }
            }
        }
    }

    /// Update a post (requires authentication, must be author)
    pub async fn update_post(
        &self,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<http_client::PostResponse, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    let http = client.lock().await;
                    http.update_post(id, title, content).await
                } else {
                    Err(BlogClientError::TransportError(
                        "HTTP client not initialized".into(),
                    ))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &self.grpc_client {
                    let grpc = client.lock().await;
                    let post = grpc.update_post(id, title, content).await?;

                    Ok(http_client::PostResponse {
                        id: post.id,
                        title: post.title,
                        content: post.content,
                        author_id: post.author_id,
                        created_at: post.created_at,
                        updated_at: post.updated_at,
                    })
                } else {
                    Err(BlogClientError::TransportError(
                        "gRPC client not initialized".into(),
                    ))
                }
            }
        }
    }

    /// Delete a post (requires authentication, must be author)
    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    let http = client.lock().await;
                    http.delete_post(id).await
                } else {
                    Err(BlogClientError::TransportError(
                        "HTTP client not initialized".into(),
                    ))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &self.grpc_client {
                    let grpc = client.lock().await;
                    grpc.delete_post(id).await
                } else {
                    Err(BlogClientError::TransportError(
                        "gRPC client not initialized".into(),
                    ))
                }
            }
        }
    }

    /// List posts with pagination
    pub async fn list_posts(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<http_client::PostsResponse, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    let http = client.lock().await;
                    http.list_posts(limit, offset).await
                } else {
                    Err(BlogClientError::TransportError(
                        "HTTP client not initialized".into(),
                    ))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &self.grpc_client {
                    let grpc = client.lock().await;

                    let page = (offset.unwrap_or(0) / limit.unwrap_or(10)) as i32 + 1;
                    let page_size = limit.unwrap_or(10) as i32;

                    let response = grpc.list_posts(page, page_size).await?;

                    Ok(http_client::PostsResponse {
                        posts: response
                            .posts
                            .into_iter()
                            .map(|p| http_client::PostResponse {
                                id: p.id,
                                title: p.title,
                                content: p.content,
                                author_id: p.author_id,
                                created_at: p.created_at,
                                updated_at: p.updated_at,
                            })
                            .collect(),
                        total: response.total_count as i64,
                        limit: limit.unwrap_or(10),
                        offset: offset.unwrap_or(0),
                    })
                } else {
                    Err(BlogClientError::TransportError(
                        "gRPC client not initialized".into(),
                    ))
                }
            }
        }
    }

    /// Check if the client is using HTTP transport
    pub fn is_http(&self) -> bool {
        matches!(self.transport, Transport::Http(_))
    }

    /// Check if the client is using gRPC transport
    pub fn is_grpc(&self) -> bool {
        matches!(self.transport, Transport::Grpc(_))
    }

    /// Get the current transport URL/address
    pub fn transport_url(&self) -> String {
        match &self.transport {
            Transport::Http(url) => url.clone(),
            Transport::Grpc(addr) => addr.clone(),
        }
    }
}
