use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::model::response_model::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Invalid session")]
    InvalidSession,
    #[error("Session has expired")]
    SessionExpired,
    #[error("Session token error")]
    SessionToken,
    #[error("Access forbidden")]
    AccessForbidden,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User blocked")]
    UserBlocked,
    #[error("Generate password hash error")]
    PasswordHash,
}

impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        let status_code = match self {
            SessionError::InvalidSession
            | SessionError::InvalidCredentials
            | SessionError::SessionExpired => StatusCode::UNAUTHORIZED,
            SessionError::PasswordHash
            | SessionError::SessionToken => StatusCode::INTERNAL_SERVER_ERROR,
            SessionError::UserBlocked
            | SessionError::AccessForbidden => StatusCode::FORBIDDEN,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
