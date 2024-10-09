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

    #[cfg_attr(test, error("sqlx error: {0}"))]
    #[cfg_attr(not(test), error("internal server error"))]
    Sqlx(#[from] sqlx::Error),
}

impl ServerError {
    pub fn as_status(&self) -> StatusCode {
        match self {
            Self::MissingAuthCode | Self::FailedRecvGitResponse | Self::FailedParseRequestBody | Self::UserRoomIsNotOpen => StatusCode::BAD_REQUEST,
            Self::InvalidSessionToken | Self::RequiredSessionToken => StatusCode::UNAUTHORIZED,
            Self::FailedConnectGithubApi | Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        #[cfg(test)]
        eprintln!("{}", self.source().unwrap_or(&self));
        Response::builder()
            .status(self.as_status())
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}