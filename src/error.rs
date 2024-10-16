use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::error::Error;
use thiserror::Error;

pub type ServerResult<T = ()> = Result<T, ServerError>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Missing auth code in query")]
    MissingAuthCode,

    #[error("Failed to connect github api")]
    FailedConnectGithubApi,

    #[error("User room is not open")]
    UserRoomIsNotOpen,

    #[error("Invalid session token")]
    InvalidSessionToken,

    #[error("Required session token")]
    RequiredSessionToken,

    #[error("Failed parse request body")]
    FailedParseRequestBody,

    #[error("Failed recv git response")]
    FailedRecvGitResponse,

    #[error("Failed parse git response")]
    FailedParseGitResponse,

    #[cfg_attr(test, error("sqlx error: {0}"))]
    #[cfg_attr(not(test), error("internal server error"))]
    Sqlx(#[from] sqlx::Error),
}

impl ServerError {
    pub fn as_status(&self) -> StatusCode {
        match self {
            Self::MissingAuthCode | Self::FailedRecvGitResponse | Self::FailedParseRequestBody => StatusCode::BAD_REQUEST,
            Self::InvalidSessionToken | Self::RequiredSessionToken => StatusCode::UNAUTHORIZED,
            Self::UserRoomIsNotOpen => StatusCode::NOT_FOUND,
            Self::FailedParseGitResponse | Self::FailedConnectGithubApi | Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status_code = self.as_status();
        if status_code == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("{}", self.source().unwrap_or(&self));
        }

        Response::builder()
            .status(status_code)
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}