use crate::models::*;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};

const API_BASE: &str = "http://localhost:3000";
const TOKEN_KEY: &str = "blog_token";

#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: API_BASE.to_string(),
        }
    }

    pub fn save_token(token: &str) {
        if let Err(e) = LocalStorage::set(TOKEN_KEY, token) {
            web_sys::console::log_1(&format!("Failed to save token: {:?}", e).into());
        }
    }

    pub fn get_token() -> Option<String> {
        LocalStorage::get(TOKEN_KEY).ok()
    }

    pub fn clear_token() {
        LocalStorage::delete(TOKEN_KEY);
    }

    fn auth_header() -> String {
        match Self::get_token() {
            Some(token) => format!("Bearer {}", token),
            None => String::new(),
        }
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<&impl Serialize>,
        requires_auth: bool,
    ) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);

        // Создаем базовый запрос в зависимости от метода
        let request_builder = match method {
            "GET" => Request::get(&url),
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "DELETE" => Request::delete(&url),
            _ => return Err(format!("Unsupported method: {}", method)),
        };

        // Добавляем заголовки
        let request_builder = request_builder.header("Content-Type", "application/json");

        let request_builder = if requires_auth {
            let auth_header = Self::auth_header();
            if !auth_header.is_empty() {
                request_builder.header("Authorization", &auth_header)
            } else {
                request_builder
            }
        } else {
            request_builder
        };

        // Создаем и отправляем запрос
        let response = if let Some(body) = body {
            // Для методов, которые могут иметь тело (POST, PUT)
            if method == "GET" || method == "DELETE" {
                return Err(format!("Method {} cannot have body", method));
            }

            let body_json = serde_json::to_string(body)
                .map_err(|e| format!("Failed to serialize request: {}", e))?;

            request_builder
                .body(body_json)
                .map_err(|e| format!("Failed to set request body: {}", e))?
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?
        } else {
            // Для методов без тела
            request_builder
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?
        };

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if (200..300).contains(&status) {
            serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            // Пытаемся распарсить сообщение об ошибке
            match serde_json::from_str::<ErrorResponse>(&text) {
                Ok(err) => Err(err.error),
                Err(_) => Err(format!("HTTP {}: {}", status, text)),
            }
        }
    }

    pub async fn register(&self, req: &RegisterRequest) -> Result<AuthResponse, String> {
        self.request("POST", "/api/auth/register", Some(req), false)
            .await
    }

    pub async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, String> {
        self.request("POST", "/api/auth/login", Some(req), false)
            .await
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<PostsResponse, String> {
        self.request(
            "GET",
            &format!("/api/posts?limit={}&offset={}", limit, offset),
            None::<&()>,
            false,
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn get_post(&self, id: i64) -> Result<Post, String> {
        self.request("GET", &format!("/api/posts/{}", id), None::<&()>, false)
            .await
    }

    pub async fn create_post(&self, req: &CreatePostRequest) -> Result<Post, String> {
        self.request("POST", "/api/protected/posts", Some(req), true)
            .await
    }

    pub async fn update_post(&self, id: i64, req: &UpdatePostRequest) -> Result<Post, String> {
        self.request(
            "PUT",
            &format!("/api/protected/posts/{}", id),
            Some(req),
            true,
        )
        .await
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), String> {
        self.request::<serde_json::Value>(
            "DELETE",
            &format!("/api/protected/posts/{}", id),
            None::<&()>,
            true,
        )
        .await?;
        Ok(())
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}
