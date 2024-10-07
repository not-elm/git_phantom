use thiserror::Error;

pub type ServerResult<T = ()> = Result<T, ServerError>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("invalid session token")]
    InvalidSessionToken,

    #[cfg_attr(test, error("sqlx error: {0}"))]
    #[cfg_attr(not(test), error("internal server error"))]
    Sqlx(#[from] sqlx::Error),
}