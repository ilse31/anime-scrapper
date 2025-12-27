//! Email service module for sending verification and password reset emails
//!
//! This module provides functionality for:
//! - Sending email verification emails
//! - Sending password reset emails

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use thiserror::Error;

use crate::config::SmtpConfig;

/// Email service errors
#[derive(Debug, Error)]
pub enum EmailError {
    #[error("SMTP transport error: {0}")]
    SmtpError(String),

    #[error("Failed to build email: {0}")]
    BuildError(String),

    #[error("Email service not configured")]
    NotConfigured,
}

/// Email service for sending transactional emails
#[derive(Clone)]
pub struct EmailService {
    config: SmtpConfig,
    frontend_url: String,
}

impl EmailService {
    /// Create a new email service
    pub fn new(config: SmtpConfig, frontend_url: String) -> Self {
        Self {
            config,
            frontend_url,
        }
    }

    /// Build SMTP transport
    fn build_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>, EmailError> {
        let creds = Credentials::new(self.config.username.clone(), self.config.password.clone());

        AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.host)
            .map_err(|e| EmailError::SmtpError(e.to_string()))?
            .credentials(creds)
            .port(self.config.port)
            .build()
            .pipe(Ok)
    }

    /// Send an email
    async fn send_email(&self, to: &str, subject: &str, body: String) -> Result<(), EmailError> {
        let from = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let email = Message::builder()
            .from(from.parse().map_err(|e| EmailError::BuildError(format!("{}", e)))?)
            .to(to.parse().map_err(|e| EmailError::BuildError(format!("{}", e)))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| EmailError::BuildError(e.to_string()))?;

        let transport = self.build_transport()?;
        transport
            .send(email)
            .await
            .map_err(|e| EmailError::SmtpError(e.to_string()))?;

        Ok(())
    }

    /// Send email verification email
    pub async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), EmailError> {
        let verification_url = format!("{}/verify-email?token={}", self.frontend_url, token);

        let body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Verify Your Email</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #2563eb;">Verify Your Email Address</h1>
        <p>Thank you for registering! Please click the button below to verify your email address:</p>
        <p style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background-color: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">
                Verify Email
            </a>
        </p>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{}</p>
        <p style="color: #666; font-size: 14px; margin-top: 30px;">
            This link will expire in 24 hours. If you didn't create an account, you can safely ignore this email.
        </p>
    </div>
</body>
</html>"#,
            verification_url, verification_url
        );

        self.send_email(to, "Verify Your Email Address", body).await
    }

    /// Send password reset email
    pub async fn send_password_reset_email(&self, to: &str, token: &str) -> Result<(), EmailError> {
        let reset_url = format!("{}/reset-password?token={}", self.frontend_url, token);

        let body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Reset Your Password</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #2563eb;">Reset Your Password</h1>
        <p>We received a request to reset your password. Click the button below to create a new password:</p>
        <p style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background-color: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">
                Reset Password
            </a>
        </p>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{}</p>
        <p style="color: #666; font-size: 14px; margin-top: 30px;">
            This link will expire in 1 hour. If you didn't request a password reset, you can safely ignore this email.
        </p>
    </div>
</body>
</html>"#,
            reset_url, reset_url
        );

        self.send_email(to, "Reset Your Password", body).await
    }
}

/// Helper trait for pipe syntax
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_error_display() {
        let err = EmailError::NotConfigured;
        assert_eq!(err.to_string(), "Email service not configured");

        let err = EmailError::SmtpError("connection failed".to_string());
        assert_eq!(err.to_string(), "SMTP transport error: connection failed");
    }
}
