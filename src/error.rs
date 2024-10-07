use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type ServerResult<T = ()> = Result<T, ServerError>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Missing auth code in query")]
    MissingAuthCode,

    #[error("Failed to connect github api")]
    FailedConnectGithubApi,

    #[error("Invalid session token")]
    InvalidSessionToken,

    #[cfg_attr(test, error("sqlx error: {0}"))]
    #[cfg_attr(not(test), error("internal server error"))]
    Sqlx(#[from] sqlx::Error),
}

impl ServerError {
    pub fn as_status(&self) -> StatusCode {
        match self {
            Self::MissingAuthCode | Self::InvalidSessionToken => StatusCode::BAD_REQUEST,
            Self::FailedConnectGithubApi | Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self.as_status())
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}