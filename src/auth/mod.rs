//! Authentication module for the Anime Scraper API
//!
//! This module provides authentication functionality including:
//! - Password hashing with bcrypt
//! - JWT token generation and verification
//! - Google OAuth token verification
//! - Authentication middleware for protected routes
//! - HTTP-only cookie support for secure token storage

use actix_web::cookie::time::Duration as CookieDuration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::dev::ServiceRequest;
use actix_web::{web, FromRequest, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use thiserror::Error;

use crate::models::ApiError;

/// Default bcrypt cost factor (12 is recommended for production)
const BCRYPT_COST: u32 = 12;

/// JWT token expiry duration in days
const JWT_EXPIRY_DAYS: i64 = 7;

/// Cookie name for JWT token
pub const AUTH_COOKIE_NAME: &str = "auth_token";

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Password hashing failed: {0}")]
    HashingError(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),

    #[error("Token verification failed: {0}")]
    TokenVerificationError(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing authorization header")]
    MissingAuthHeader,

    #[error("Invalid authorization header format")]
    InvalidAuthHeaderFormat,

    #[error("Google OAuth verification failed: {0}")]
    GoogleOAuthError(String),

    #[error("User not found")]
    UserNotFound,
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: i32,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at time (Unix timestamp)
    pub iat: i64,
}

/// Google OAuth token payload (subset of fields we need)
#[derive(Debug, Deserialize)]
pub struct GoogleTokenPayload {
    /// Google user ID
    pub sub: String,
    /// User email
    pub email: String,
    /// Whether email is verified
    pub email_verified: Option<bool>,
    /// User's full name
    pub name: Option<String>,
    /// User's profile picture URL
    pub picture: Option<String>,
}

/// Authenticated user info extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// User ID from the JWT
    pub user_id: i32,
}

/// Hash a password using bcrypt
///
/// # Arguments
/// * `password` - The plain text password to hash
///
/// # Returns
/// * `Ok(String)` - The hashed password
/// * `Err(AuthError)` - If hashing fails
///
/// # Example
/// ```ignore
/// let hash = hash_password("my_secure_password")?;
/// ```
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    bcrypt::hash(password, BCRYPT_COST).map_err(|e| AuthError::HashingError(e.to_string()))
}

/// Verify a password against a bcrypt hash
///
/// # Arguments
/// * `password` - The plain text password to verify
/// * `hash` - The bcrypt hash to verify against
///
/// # Returns
/// * `Ok(true)` - If the password matches
/// * `Ok(false)` - If the password doesn't match
/// * `Err(AuthError)` - If verification fails
///
/// # Example
/// ```ignore
/// let is_valid = verify_password("my_password", &stored_hash)?;
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    bcrypt::verify(password, hash).map_err(|e| AuthError::HashingError(e.to_string()))
}

/// Generate a JWT token for a user
///
/// # Arguments
/// * `user_id` - The user's ID to encode in the token
/// * `secret` - The JWT secret key for signing
///
/// # Returns
/// * `Ok(String)` - The generated JWT token
/// * `Err(AuthError)` - If token generation fails
///
/// # Example
/// ```ignore
/// let token = generate_token(user_id, &jwt_secret)?;
/// ```
pub fn generate_token(user_id: i32, secret: &str) -> Result<String, AuthError> {
    let now = Utc::now();
    let expiry = now + Duration::days(JWT_EXPIRY_DAYS);

    let claims = Claims {
        sub: user_id,
        exp: expiry.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
}

/// Verify and decode a JWT token
///
/// # Arguments
/// * `token` - The JWT token to verify
/// * `secret` - The JWT secret key for verification
///
/// # Returns
/// * `Ok(Claims)` - The decoded claims if valid
/// * `Err(AuthError)` - If verification fails or token is expired
///
/// # Example
/// ```ignore
/// let claims = verify_token(&token, &jwt_secret)?;
/// let user_id = claims.sub;
/// ```
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AuthError> {
    let token_data: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        _ => AuthError::TokenVerificationError(e.to_string()),
    })?;

    Ok(token_data.claims)
}

// ============================================================================
// HTTP-Only Cookie Management
// ============================================================================

/// Create an HTTP-only cookie containing the JWT token
///
/// # Arguments
/// * `token` - The JWT token to store in the cookie
///
/// # Returns
/// A Cookie configured with:
/// - HttpOnly: true (prevents JavaScript access)
/// - Secure: true (only sent over HTTPS in production)
/// - SameSite: Lax (CSRF protection)
/// - Path: "/" (available for all routes)
/// - Max-Age: 7 days (matches JWT expiry)
pub fn create_auth_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(AUTH_COOKIE_NAME, token.to_owned())
        .path("/")
        .http_only(true)
        .secure(true) // Set to false for local development without HTTPS
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::days(JWT_EXPIRY_DAYS))
        .finish()
}

/// Create a cookie that clears the auth token (for logout)
///
/// # Returns
/// A Cookie configured to expire immediately, effectively removing the auth cookie
pub fn create_logout_cookie() -> Cookie<'static> {
    Cookie::build(AUTH_COOKIE_NAME, "")
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::ZERO)
        .finish()
}

/// Extract JWT token from cookie
///
/// # Arguments
/// * `req` - The HTTP request
///
/// # Returns
/// * `Some(&str)` - The token if found in cookies
/// * `None` - If no auth cookie is present
pub fn extract_token_from_cookie(req: &HttpRequest) -> Option<String> {
    req.cookie(AUTH_COOKIE_NAME).map(|c| c.value().to_owned())
}

/// Verify a Google ID token and extract user info
///
/// # Arguments
/// * `id_token` - The Google ID token from the client
/// * `_client_id` - The Google OAuth client ID (reserved for future signature verification)
///
/// # Returns
/// * `Ok(GoogleTokenPayload)` - The decoded user info if valid
/// * `Err(AuthError)` - If verification fails
///
/// # Note
/// This function verifies the token by calling Google's tokeninfo endpoint.
/// In production, you might want to use the google-auth library for proper
/// signature verification.
pub async fn verify_google_token(
    id_token: &str,
    _client_id: &str,
) -> Result<GoogleTokenPayload, AuthError> {
    // Use Google's tokeninfo endpoint to verify the token
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        id_token
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AuthError::GoogleOAuthError(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AuthError::GoogleOAuthError("Invalid token".to_string()));
    }

    let payload: GoogleTokenPayload = response
        .json()
        .await
        .map_err(|e| AuthError::GoogleOAuthError(format!("Failed to parse response: {}", e)))?;

    // Note: In a production environment, you should also verify:
    // 1. The 'aud' (audience) claim matches your client_id
    // 2. The 'iss' (issuer) claim is accounts.google.com or https://accounts.google.com
    // For now, we trust Google's tokeninfo endpoint validation

    Ok(payload)
}

/// Extract JWT token from Authorization header
///
/// # Arguments
/// * `auth_header` - The Authorization header value
///
/// # Returns
/// * `Ok(&str)` - The extracted token
/// * `Err(AuthError)` - If the header format is invalid
pub fn extract_token_from_header(auth_header: &str) -> Result<&str, AuthError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidAuthHeaderFormat);
    }

    let token = auth_header.trim_start_matches("Bearer ").trim();
    if token.is_empty() {
        return Err(AuthError::InvalidAuthHeaderFormat);
    }

    Ok(token)
}

/// Validate a request and extract the authenticated user
///
/// This function extracts the JWT from the Authorization header,
/// verifies it, and returns the authenticated user info.
///
/// # Arguments
/// * `req` - The service request
/// * `secret` - The JWT secret key
///
/// # Returns
/// * `Ok(AuthenticatedUser)` - The authenticated user info
/// * `Err(AuthError)` - If authentication fails
pub fn validate_request(
    req: &ServiceRequest,
    secret: &str,
) -> Result<AuthenticatedUser, AuthError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingAuthHeader)?;

    let token = extract_token_from_header(auth_header)?;
    let claims = verify_token(token, secret)?;

    Ok(AuthenticatedUser {
        user_id: claims.sub,
    })
}

/// Validate an HTTP request and extract the authenticated user
///
/// This function checks for JWT token in the following order:
/// 1. Authorization header (Bearer token)
/// 2. HTTP-only cookie (auth_token)
///
/// # Arguments
/// * `req` - The HTTP request
/// * `secret` - The JWT secret key
///
/// # Returns
/// * `Ok(AuthenticatedUser)` - The authenticated user info
/// * `Err(AuthError)` - If authentication fails
pub fn validate_http_request(
    req: &HttpRequest,
    secret: &str,
) -> Result<AuthenticatedUser, AuthError> {
    // First, try to get token from Authorization header
    let token = if let Some(auth_header) = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
    {
        extract_token_from_header(auth_header)?.to_owned()
    } else if let Some(cookie_token) = extract_token_from_cookie(req) {
        // Fall back to cookie if no Authorization header
        cookie_token
    } else {
        return Err(AuthError::MissingAuthHeader);
    };

    let claims = verify_token(&token, secret)?;

    Ok(AuthenticatedUser {
        user_id: claims.sub,
    })
}

/// Configuration for the auth extractor
#[derive(Clone)]
pub struct AuthConfig {
    /// JWT secret key
    pub jwt_secret: String,
}

/// Authenticated user extractor for Actix-web routes
///
/// This extractor can be used in route handlers to require authentication.
/// It extracts the JWT from the Authorization header, verifies it, and
/// provides the authenticated user info.
///
/// # Example
/// ```ignore
/// async fn protected_route(user: Auth) -> impl Responder {
///     HttpResponse::Ok().json(format!("Hello, user {}", user.user_id))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Auth {
    /// The authenticated user's ID
    pub user_id: i32,
}

impl FromRequest for Auth {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        // Get the JWT secret from app data
        let config = req.app_data::<web::Data<AuthConfig>>();

        let result = match config {
            Some(config) => match validate_http_request(req, &config.jwt_secret) {
                Ok(user) => Ok(Auth {
                    user_id: user.user_id,
                }),
                Err(e) => {
                    let error_response = match &e {
                        AuthError::MissingAuthHeader => HttpResponse::Unauthorized()
                            .json(ApiError::new("Missing authorization header")),
                        AuthError::InvalidAuthHeaderFormat => HttpResponse::Unauthorized()
                            .json(ApiError::new("Invalid authorization header format")),
                        AuthError::TokenExpired => {
                            HttpResponse::Unauthorized().json(ApiError::new("Token expired"))
                        }
                        AuthError::TokenVerificationError(_) | AuthError::InvalidToken => {
                            HttpResponse::Unauthorized().json(ApiError::new("Invalid token"))
                        }
                        _ => HttpResponse::Unauthorized()
                            .json(ApiError::new("Authentication failed")),
                    };
                    Err(actix_web::error::InternalError::from_response(e, error_response).into())
                }
            },
            None => {
                let error_response = HttpResponse::InternalServerError()
                    .json(ApiError::new("Auth configuration not found"));
                Err(actix_web::error::InternalError::from_response(
                    AuthError::TokenVerificationError("Config not found".to_string()),
                    error_response,
                )
                .into())
            }
        };

        ready(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_creates_valid_hash() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        // Hash should not be empty
        assert!(!hash.is_empty());
        // Hash should start with bcrypt identifier
        assert!(hash.starts_with("$2"));
        // Hash should be different from password
        assert_ne!(hash, password);
    }

    #[test]
    fn test_hash_password_different_hashes_for_same_password() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Due to salt, hashes should be different
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_password_correct_password() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect_password() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).unwrap();

        let result = verify_password(wrong_password, &hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_verify_password_empty_password() {
        let password = "";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_unicode_password() {
        let password = "пароль_密码_🔐";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_generate_token_creates_valid_token() {
        let user_id = 42;
        let secret = "test_secret_key";

        let token = generate_token(user_id, secret).unwrap();

        // Token should not be empty
        assert!(!token.is_empty());
        // Token should have 3 parts (header.payload.signature)
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_verify_token_valid_token() {
        let user_id = 123;
        let secret = "test_secret_key";

        let token = generate_token(user_id, secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let user_id = 123;
        let secret = "correct_secret";
        let wrong_secret = "wrong_secret";

        let token = generate_token(user_id, secret).unwrap();
        let result = verify_token(&token, wrong_secret);

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_invalid_token() {
        let secret = "test_secret";
        let invalid_token = "invalid.token.here";

        let result = verify_token(invalid_token, secret);

        assert!(result.is_err());
    }

    #[test]
    fn test_token_contains_correct_claims() {
        let user_id = 999;
        let secret = "test_secret";

        let token = generate_token(user_id, secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id);
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
        // Expiry should be approximately 7 days from now
        let expected_expiry = claims.iat + (7 * 24 * 60 * 60);
        assert!((claims.exp - expected_expiry).abs() < 60); // Within 1 minute tolerance
    }

    #[test]
    fn test_extract_token_valid_header() {
        let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test";
        let token = extract_token_from_header(header).unwrap();

        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test");
    }

    #[test]
    fn test_extract_token_missing_bearer() {
        let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test";
        let result = extract_token_from_header(header);

        assert!(matches!(result, Err(AuthError::InvalidAuthHeaderFormat)));
    }

    #[test]
    fn test_extract_token_empty_token() {
        let header = "Bearer ";
        let result = extract_token_from_header(header);

        assert!(matches!(result, Err(AuthError::InvalidAuthHeaderFormat)));
    }

    #[test]
    fn test_extract_token_lowercase_bearer() {
        let header = "bearer token123";
        let result = extract_token_from_header(header);

        // Should fail because "Bearer" is case-sensitive
        assert!(matches!(result, Err(AuthError::InvalidAuthHeaderFormat)));
    }

    #[test]
    fn test_extract_token_with_extra_spaces() {
        let header = "Bearer   token123  ";
        let token = extract_token_from_header(header).unwrap();

        assert_eq!(token, "token123");
    }

    // ========================================================================
    // HTTP-Only Cookie Tests
    // ========================================================================

    #[test]
    fn test_create_auth_cookie_properties() {
        let token = "test_jwt_token_123";
        let cookie = create_auth_cookie(token);

        assert_eq!(cookie.name(), AUTH_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert!(cookie.http_only().unwrap_or(false));
        assert!(cookie.secure().unwrap_or(false));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[test]
    fn test_create_logout_cookie_clears_value() {
        let cookie = create_logout_cookie();

        assert_eq!(cookie.name(), AUTH_COOKIE_NAME);
        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.path(), Some("/"));
        assert!(cookie.http_only().unwrap_or(false));
        // Max-age should be zero to expire immediately
        assert_eq!(cookie.max_age(), Some(CookieDuration::ZERO));
    }

    // ========================================================================
    // AuthError Display Tests
    // ========================================================================

    #[test]
    fn test_auth_error_display() {
        assert_eq!(
            AuthError::InvalidCredentials.to_string(),
            "Invalid credentials"
        );
        assert_eq!(AuthError::TokenExpired.to_string(), "Token expired");
        assert_eq!(
            AuthError::MissingAuthHeader.to_string(),
            "Missing authorization header"
        );
    }
}
