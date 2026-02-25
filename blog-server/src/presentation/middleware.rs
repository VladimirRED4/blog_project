use crate::infrastructure::jwt::JwtService;
use actix_web::{dev::ServiceRequest, web, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use std::sync::Arc;

pub async fn jwt_middleware(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = match req.app_data::<web::Data<Arc<JwtService>>>() {
        Some(service) => service.get_ref().clone(),
        None => {
            return Err((
                actix_web::error::ErrorInternalServerError("JWT service not configured"),
                req,
            ));
        }
    };

    // Verify token
    match jwt_service.verify_token(credentials.token()) {
        Ok(user_id) => {
            req.extensions_mut().insert(user_id);
            Ok(req)
        }
        Err(_) => {
            let config = req.app_data::<Config>().cloned().unwrap_or_default();
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}
