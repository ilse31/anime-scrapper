//! Authentication routes for the Anime Scraper API
//!
//! This module contains HTTP route handlers for authentication endpoints:
//! - POST /api/auth/register - Register with email/password
//! - POST /api/auth/login - Login with email/password
//! - POST /api/auth/google - Login with Google OAuth
//! - POST /api/auth/logout - Logout (clears HTTP-only cookie)
//! - GET /api/auth/me - Get current user info
//! - POST /api/auth/forgot-password - Request password reset email
//! - POST /api/auth/reset-password - Reset password with token
//! - POST /api/auth/verify-email - Verify email with token
//! - POST /api/auth/resend-verification - Resend verification email

use actix_web::{web, HttpResponse, Responder};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::auth::{
    create_auth_cookie, create_logout_cookie, generate_token, hash_password, verify_google_token,
    verify_password, Auth,
};
use crate::db::{
    create_google_user, create_user, create_verification_token, delete_user_tokens,
    find_user_by_email, find_user_by_google_id, find_user_by_id, find_verification_token,
    link_google_account, mark_token_as_used, set_email_verified, update_user_password,
    RepositoryError, TOKEN_TYPE_EMAIL_VERIFICATION, TOKEN_TYPE_PASSWORD_RESET,
};
use crate::models::{
    ApiError, ApiResponse, AuthData, AuthResponse, ForgotPasswordRequest, GoogleAuthRequest,
    LoginRequest, RegisterRequest, ResendVerificationRequest, ResetPasswordRequest, User,
    VerifyEmailRequest,
};
use crate::routes::AppState;

/// Simple email validation using basic regex pattern
fn is_valid_email(email: &str) -> bool {
    // Basic email validation: contains @ and at least one . after @
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];

    // Local part must not be empty
    if local.is_empty() {
        return false;
    }

    // Domain must contain at least one dot and not be empty
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }

    // Domain parts must not be empty
    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.iter().any(|p| p.is_empty()) {
        return false;
    }

    true
}

/// POST /api/auth/register - Register a new user with email and password
///
/// # Request Body
/// - email: User's email address (required, must be valid format)
/// - password: User's password (required)
/// - name: Optional display name
///
/// # Responses
/// - 200: Registration successful, returns user info and JWT token
/// - 400: Invalid email format or missing required fields
/// - 409: Email already exists
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = AuthResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 409, description = "Email already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn register(
    data: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Validate email format
    if !is_valid_email(&body.email) {
        return HttpResponse::BadRequest().json(ApiError::new("Invalid email format"));
    }

    // Validate password is not empty
    if body.password.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Password is required"));
    }

    // Hash the password
    let password_hash = match hash_password(&body.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process registration"));
        }
    };

    // Create the user
    let user = match create_user(pool, &body.email, &password_hash, body.name.as_deref()).await {
        Ok(user) => user,
        Err(RepositoryError::EmailAlreadyExists) => {
            return HttpResponse::Conflict().json(ApiError::new("Email already exists"));
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to create user"));
        }
    };

    info!("User registered: {}", user.email);

    // Generate JWT token
    let token = match generate_token(user.id, &data.config.jwt_secret) {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate token: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to generate authentication token"));
        }
    };

    // Create HTTP-only cookie with the token
    let cookie = create_auth_cookie(&token);

    HttpResponse::Ok().cookie(cookie).json(AuthResponse {
        success: true,
        data: AuthData { user, token },
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// POST /api/auth/login - Login with email and password
///
/// # Request Body
/// - email: User's email address (required)
/// - password: User's password (required)
///
/// # Responses
/// - 200: Login successful, returns user info and JWT token
/// - 400: Missing required fields
/// - 401: Invalid credentials
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 401, description = "Invalid credentials", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn login(data: web::Data<AppState>, body: web::Json<LoginRequest>) -> impl Responder {
    let pool = data.db.pool();

    // Validate required fields
    if body.email.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Email is required"));
    }

    if body.password.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Password is required"));
    }

    // Find user by email
    let (user, password_hash) = match find_user_by_email(pool, &body.email).await {
        Ok(Some((user, hash))) => (user, hash),
        Ok(None) => {
            return HttpResponse::Unauthorized().json(ApiError::new("Invalid credentials"));
        }
        Err(e) => {
            error!("Failed to find user: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process login"));
        }
    };

    // Check if user has a password (not a Google-only account)
    let password_hash = match password_hash {
        Some(hash) => hash,
        None => {
            return HttpResponse::Unauthorized().json(ApiError::new("Invalid credentials"));
        }
    };

    // Verify password
    match verify_password(&body.password, &password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Unauthorized().json(ApiError::new("Invalid credentials"));
        }
        Err(e) => {
            error!("Failed to verify password: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process login"));
        }
    }

    info!("User logged in: {}", user.email);

    // Generate JWT token
    let token = match generate_token(user.id, &data.config.jwt_secret) {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate token: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to generate authentication token"));
        }
    };

    // Create HTTP-only cookie with the token
    let cookie = create_auth_cookie(&token);

    HttpResponse::Ok().cookie(cookie).json(AuthResponse {
        success: true,
        data: AuthData { user, token },
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// POST /api/auth/google - Login or register with Google OAuth
///
/// # Request Body
/// - idToken: Google ID token from client (required)
///
/// # Responses
/// - 200: Authentication successful, returns user info and JWT token
/// - 400: Missing or invalid ID token
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/auth/google",
    tag = "auth",
    request_body = GoogleAuthRequest,
    responses(
        (status = 200, description = "Authentication successful", body = AuthResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn google_auth(
    data: web::Data<AppState>,
    body: web::Json<GoogleAuthRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Check if Google OAuth is configured
    let google_client_id = match &data.config.google_client_id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest()
                .json(ApiError::new("Google OAuth is not configured"));
        }
    };

    // Validate ID token is not empty
    if body.id_token.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("ID token is required"));
    }

    // Verify Google ID token
    let google_payload = match verify_google_token(&body.id_token, google_client_id).await {
        Ok(payload) => payload,
        Err(e) => {
            warn!("Google token verification failed: {}", e);
            return HttpResponse::BadRequest().json(ApiError::new("Invalid Google ID token"));
        }
    };

    // Try to find existing user by Google ID
    let user = match find_user_by_google_id(pool, &google_payload.sub).await {
        Ok(Some(user)) => {
            info!("Existing Google user logged in: {}", user.email);
            user
        }
        Ok(None) => {
            // Check if user exists with this email (link accounts)
            match find_user_by_email(pool, &google_payload.email).await {
                Ok(Some((existing_user, _))) => {
                    // Link Google account to existing user
                    if let Err(e) =
                        link_google_account(pool, existing_user.id, &google_payload.sub).await
                    {
                        error!("Failed to link Google account: {}", e);
                        return HttpResponse::InternalServerError()
                            .json(ApiError::new("Failed to link Google account"));
                    }
                    info!(
                        "Linked Google account to existing user: {}",
                        existing_user.email
                    );
                    existing_user
                }
                Ok(None) => {
                    // Create new user with Google OAuth
                    match create_google_user(
                        pool,
                        &google_payload.email,
                        &google_payload.sub,
                        google_payload
                            .name
                            .as_deref()
                            .unwrap_or(&google_payload.email),
                        google_payload.picture.as_deref(),
                    )
                    .await
                    {
                        Ok(user) => {
                            info!("New Google user created: {}", user.email);
                            user
                        }
                        Err(e) => {
                            error!("Failed to create Google user: {}", e);
                            return HttpResponse::InternalServerError()
                                .json(ApiError::new("Failed to create user"));
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to find user by email: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiError::new("Failed to process authentication"));
                }
            }
        }
        Err(e) => {
            error!("Failed to find user by Google ID: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process authentication"));
        }
    };

    // Generate JWT token
    let token = match generate_token(user.id, &data.config.jwt_secret) {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate token: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to generate authentication token"));
        }
    };

    // Create HTTP-only cookie with the token
    let cookie = create_auth_cookie(&token);

    HttpResponse::Ok().cookie(cookie).json(AuthResponse {
        success: true,
        data: AuthData { user, token },
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// POST /api/auth/logout - Logout (clears HTTP-only cookie)
///
/// This endpoint clears the HTTP-only authentication cookie.
/// The server doesn't maintain a token blacklist, so the client
/// should also discard any stored token upon receiving a successful response.
///
/// # Responses
/// - 200: Logout successful
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logout successful", body = ApiResponse<String>)
    )
)]
pub async fn logout() -> impl Responder {
    // Clear the HTTP-only cookie by setting it to expire immediately
    let cookie = create_logout_cookie();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(ApiResponse::new("Logged out successfully".to_string()))
}

/// GET /api/auth/me - Get current authenticated user info
///
/// Requires a valid JWT token in the Authorization header or HTTP-only cookie.
///
/// # Responses
/// - 200: Returns current user info
/// - 401: Not authenticated or invalid token
/// - 500: Internal server error
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user info", body = ApiResponse<User>),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_me(data: web::Data<AppState>, auth: Auth) -> impl Responder {
    let pool = data.db.pool();

    // Find user by ID from JWT
    match find_user_by_id(pool, auth.user_id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse::new(user)),
        Ok(None) => HttpResponse::Unauthorized().json(ApiError::new("User not found")),
        Err(e) => {
            error!("Failed to find user: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to get user info"))
        }
    }
}

/// POST /api/auth/forgot-password - Request password reset email
///
/// # Request Body
/// - email: User's email address (required)
///
/// # Responses
/// - 200: Password reset email sent (always returns success for security)
/// - 400: Invalid email format
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    tag = "auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = ApiResponse<String>),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn forgot_password(
    data: web::Data<AppState>,
    body: web::Json<ForgotPasswordRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Validate email format
    if !is_valid_email(&body.email) {
        return HttpResponse::BadRequest().json(ApiError::new("Invalid email format"));
    }

    // Check if email service is configured
    let email_service = match &data.email_service {
        Some(service) => service,
        None => {
            error!("Email service not configured");
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Email service not available"));
        }
    };

    // Find user by email (don't reveal if user exists for security)
    let user = match find_user_by_email(pool, &body.email).await {
        Ok(Some((user, _))) => user,
        Ok(None) => {
            // Return success even if user doesn't exist (security best practice)
            info!(
                "Password reset requested for non-existent email: {}",
                body.email
            );
            return HttpResponse::Ok().json(ApiResponse::new(
                "If the email exists, a password reset link has been sent".to_string(),
            ));
        }
        Err(e) => {
            error!("Failed to find user: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process request"));
        }
    };

    // Delete any existing password reset tokens for this user
    if let Err(e) = delete_user_tokens(pool, user.id, TOKEN_TYPE_PASSWORD_RESET).await {
        warn!("Failed to delete existing tokens: {}", e);
    }

    // Generate a new token
    let token = Uuid::new_v4().to_string();

    // Create verification token (expires in 1 hour)
    if let Err(e) =
        create_verification_token(pool, user.id, &token, TOKEN_TYPE_PASSWORD_RESET, 1).await
    {
        error!("Failed to create password reset token: {}", e);
        return HttpResponse::InternalServerError()
            .json(ApiError::new("Failed to process request"));
    }

    // Send password reset email
    if let Err(e) = email_service
        .send_password_reset_email(&body.email, &token)
        .await
    {
        error!("Failed to send password reset email: {}", e);
        return HttpResponse::InternalServerError().json(ApiError::new("Failed to send email"));
    }

    info!("Password reset email sent to: {}", body.email);
    HttpResponse::Ok().json(ApiResponse::new(
        "If the email exists, a password reset link has been sent".to_string(),
    ))
}

/// POST /api/auth/reset-password - Reset password with token
///
/// # Request Body
/// - token: Password reset token (required)
/// - newPassword: New password (required)
///
/// # Responses
/// - 200: Password reset successful
/// - 400: Invalid or expired token, or invalid password
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    tag = "auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = ApiResponse<String>),
        (status = 400, description = "Invalid or expired token", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn reset_password(
    data: web::Data<AppState>,
    body: web::Json<ResetPasswordRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Validate new password
    if body.new_password.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Password is required"));
    }

    if body.new_password.len() < 6 {
        return HttpResponse::BadRequest()
            .json(ApiError::new("Password must be at least 6 characters"));
    }

    // Find the token
    let verification_token = match find_verification_token(pool, &body.token).await {
        Ok(Some(token)) => token,
        Ok(None) => {
            return HttpResponse::BadRequest().json(ApiError::new("Invalid or expired token"));
        }
        Err(e) => {
            error!("Failed to find token: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process request"));
        }
    };

    // Check token type
    if verification_token.token_type != TOKEN_TYPE_PASSWORD_RESET {
        return HttpResponse::BadRequest().json(ApiError::new("Invalid token type"));
    }

    // Check if token is expired
    if verification_token.expires_at < chrono::Utc::now() {
        return HttpResponse::BadRequest().json(ApiError::new("Token has expired"));
    }

    // Check if token was already used
    if verification_token.used_at.is_some() {
        return HttpResponse::BadRequest().json(ApiError::new("Token has already been used"));
    }

    // Hash the new password
    let password_hash = match hash_password(&body.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process request"));
        }
    };

    // Update user's password
    if let Err(e) = update_user_password(pool, verification_token.user_id, &password_hash).await {
        error!("Failed to update password: {}", e);
        return HttpResponse::InternalServerError()
            .json(ApiError::new("Failed to update password"));
    }

    // Mark token as used
    if let Err(e) = mark_token_as_used(pool, &body.token).await {
        warn!("Failed to mark token as used: {}", e);
    }

    info!(
        "Password reset successful for user_id: {}",
        verification_token.user_id
    );
    HttpResponse::Ok().json(ApiResponse::new("Password reset successful".to_string()))
}

/// POST /api/auth/verify-email - Verify email with token
///
/// # Request Body
/// - token: Email verification token (required)
///
/// # Responses
/// - 200: Email verified successfully
/// - 400: Invalid or expired token
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/auth/verify-email",
    tag = "auth",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = ApiResponse<String>),
        (status = 400, description = "Invalid or expired token", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn verify_email(
    data: web::Data<AppState>,
    body: web::Json<VerifyEmailRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Find the token
    let verification_token = match find_verification_token(pool, &body.token).await {
        Ok(Some(token)) => token,
        Ok(None) => {
            return HttpResponse::BadRequest().json(ApiError::new("Invalid or expired token"));
        }
        Err(e) => {
            error!("Failed to find token: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process request"));
        }
    };

    // Check token type
    if verification_token.token_type != TOKEN_TYPE_EMAIL_VERIFICATION {
        return HttpResponse::BadRequest().json(ApiError::new("Invalid token type"));
    }

    // Check if token is expired
    if verification_token.expires_at < chrono::Utc::now() {
        return HttpResponse::BadRequest().json(ApiError::new("Token has expired"));
    }

    // Check if token was already used
    if verification_token.used_at.is_some() {
        return HttpResponse::BadRequest().json(ApiError::new("Token has already been used"));
    }

    // Set email as verified
    if let Err(e) = set_email_verified(pool, verification_token.user_id, true).await {
        error!("Failed to verify email: {}", e);
        return HttpResponse::InternalServerError().json(ApiError::new("Failed to verify email"));
    }

    // Mark token as used
    if let Err(e) = mark_token_as_used(pool, &body.token).await {
        warn!("Failed to mark token as used: {}", e);
    }

    info!("Email verified for user_id: {}", verification_token.user_id);
    HttpResponse::Ok().json(ApiResponse::new("Email verified successfully".to_string()))
}

/// POST /api/auth/resend-verification - Resend verification email
///
/// # Request Body
/// - email: User's email address (required)
///
/// # Responses
/// - 200: Verification email sent (always returns success for security)
/// - 400: Invalid email format
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/auth/resend-verification",
    tag = "auth",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification email sent", body = ApiResponse<String>),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn resend_verification(
    data: web::Data<AppState>,
    body: web::Json<ResendVerificationRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Validate email format
    if !is_valid_email(&body.email) {
        return HttpResponse::BadRequest().json(ApiError::new("Invalid email format"));
    }

    // Check if email service is configured
    let email_service = match &data.email_service {
        Some(service) => service,
        None => {
            error!("Email service not configured");
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Email service not available"));
        }
    };

    // Find user by email
    let user = match find_user_by_email(pool, &body.email).await {
        Ok(Some((user, _))) => user,
        Ok(None) => {
            // Return success even if user doesn't exist (security best practice)
            info!(
                "Verification email requested for non-existent email: {}",
                body.email
            );
            return HttpResponse::Ok().json(ApiResponse::new(
                "If the email exists and is not verified, a verification link has been sent"
                    .to_string(),
            ));
        }
        Err(e) => {
            error!("Failed to find user: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to process request"));
        }
    };

    // Delete any existing verification tokens for this user
    if let Err(e) = delete_user_tokens(pool, user.id, TOKEN_TYPE_EMAIL_VERIFICATION).await {
        warn!("Failed to delete existing tokens: {}", e);
    }

    // Generate a new token
    let token = Uuid::new_v4().to_string();

    // Create verification token (expires in 24 hours)
    if let Err(e) =
        create_verification_token(pool, user.id, &token, TOKEN_TYPE_EMAIL_VERIFICATION, 24).await
    {
        error!("Failed to create verification token: {}", e);
        return HttpResponse::InternalServerError()
            .json(ApiError::new("Failed to process request"));
    }

    // Send verification email
    if let Err(e) = email_service
        .send_verification_email(&body.email, &token)
        .await
    {
        error!("Failed to send verification email: {}", e);
        return HttpResponse::InternalServerError().json(ApiError::new("Failed to send email"));
    }

    info!("Verification email sent to: {}", body.email);
    HttpResponse::Ok().json(ApiResponse::new(
        "If the email exists and is not verified, a verification link has been sent".to_string(),
    ))
}

/// Configure authentication routes
pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/google", web::post().to(google_auth))
            .route("/logout", web::post().to(logout))
            .route("/me", web::get().to(get_me))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
            .route("/verify-email", web::post().to(verify_email))
            .route("/resend-verification", web::post().to(resend_verification)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_email_valid() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.uk"));
        assert!(is_valid_email("user+tag@example.org"));
        assert!(is_valid_email("a@b.co"));
    }

    #[test]
    fn test_is_valid_email_invalid() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
        assert!(!is_valid_email("test@.com"));
        assert!(!is_valid_email("test@example"));
        assert!(!is_valid_email("test@@example.com"));
        assert!(!is_valid_email("test@example..com"));
    }
}
