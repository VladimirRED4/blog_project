use crate::data::user_repository::UserRepository;
use crate::domain::user::{LoginUserRequest, RegisterUserRequest, UserResponse};
use crate::domain::DomainError;
use crate::infrastructure::jwt::JwtService;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use std::sync::Arc;

pub struct AuthService {
    user_repo: Arc<dyn UserRepository + Send + Sync>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository + Send + Sync>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            user_repo,
            jwt_service,
        }
    }

    pub async fn register(
        &self,
        req: RegisterUserRequest,
    ) -> Result<(String, UserResponse), DomainError> {
        tracing::debug!("=== REGISTRATION START ===");
        tracing::debug!("Username: {}, Email: {}", req.username, req.email);

        // Check if user already exists
        tracing::debug!("Checking if username exists...");
        if let Ok(_user) = self.user_repo.find_by_username(&req.username).await {
            tracing::warn!("Registration failed: username already exists");
            return Err(DomainError::UserAlreadyExists);
        }

        tracing::debug!("Checking if email exists...");
        if let Ok(_user) = self.user_repo.find_by_email(&req.email).await {
            tracing::warn!("Registration failed: email already exists");
            return Err(DomainError::UserAlreadyExists);
        }

        // Hash password
        tracing::debug!("Hashing password...");
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = match argon2.hash_password(req.password.as_bytes(), &salt) {
            Ok(hash) => {
                tracing::debug!("Password hashed successfully");
                hash.to_string()
            }
            Err(e) => {
                tracing::error!("Password hashing failed: {}", e);
                return Err(DomainError::InternalError(format!(
                    "Password hashing failed: {}",
                    e
                )));
            }
        };

        // Create user
        tracing::debug!("Creating user in database...");
        let user = match self.user_repo.create(req, password_hash).await {
            Ok(u) => {
                tracing::debug!("User created with ID: {}", u.id);
                u
            }
            Err(e) => {
                tracing::error!("Failed to create user in database: {:?}", e);
                return Err(e);
            }
        };

        // Generate JWT token
        tracing::debug!("Generating JWT token for user ID: {}", user.id);
        tracing::debug!("JWT Service available: true");

        match self
            .jwt_service
            .generate_token(user.id, user.username.clone())
        {
            Ok(token) => {
                tracing::debug!("JWT token generated successfully");
                tracing::debug!("Token length: {}", token.len());
                tracing::info!(
                    "User registered successfully: id={}, username={}",
                    user.id,
                    user.username
                );
                Ok((token, UserResponse::from(user)))
            }
            Err(e) => {
                tracing::error!("JWT GENERATION FAILED: {:?}", e);
                tracing::error!("Error type: {:?}", std::any::type_name_of_val(&e));
                Err(e)
            }
        }
    }

    pub async fn login(
        &self,
        req: LoginUserRequest,
    ) -> Result<(String, UserResponse), DomainError> {
        tracing::debug!("=== LOGIN START ===");
        tracing::debug!("Username: {}", req.username);

        // Find user by username
        tracing::debug!("Finding user in database...");
        let user = match self.user_repo.find_by_username(&req.username).await {
            Ok(u) => {
                tracing::debug!("User found with ID: {}", u.id);
                u
            }
            Err(e) => {
                tracing::warn!("User not found: {}", req.username);
                return Err(e);
            }
        };

        // Verify password
        tracing::debug!("Verifying password...");
        let parsed_hash = match PasswordHash::new(&user.password_hash) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Invalid password hash format: {}", e);
                return Err(DomainError::InternalError(format!(
                    "Invalid password hash: {}",
                    e
                )));
            }
        };

        let argon2 = Argon2::default();
        match argon2.verify_password(req.password.as_bytes(), &parsed_hash) {
            Ok(_) => {
                tracing::debug!("Password verified successfully");
            }
            Err(_) => {
                tracing::warn!("Invalid password for user {}", user.username);
                return Err(DomainError::InvalidCredentials);
            }
        };

        // Generate JWT token
        tracing::debug!("Generating JWT token for user ID: {}", user.id);

        match self
            .jwt_service
            .generate_token(user.id, user.username.clone())
        {
            Ok(token) => {
                tracing::debug!("JWT token generated successfully");
                tracing::info!(
                    "User logged in successfully: id={}, username={}",
                    user.id,
                    user.username
                );
                Ok((token, UserResponse::from(user)))
            }
            Err(e) => {
                tracing::error!("JWT GENERATION FAILED: {:?}", e);
                Err(e)
            }
        }
    }

    #[allow(dead_code)]
    pub async fn validate_token(&self, token: &str) -> Result<i64, DomainError> {
        tracing::debug!("Validating token...");
        self.jwt_service.verify_token(token).map_err(|e| {
            tracing::warn!("Token validation failed: {:?}", e);
            DomainError::Unauthorized("Invalid token".to_string())
        })
    }
}
