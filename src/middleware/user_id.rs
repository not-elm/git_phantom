use crate::db::users::UsersTable;
use crate::error::ServerError;
use crate::middleware::session_token::SessionToken;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Default)]
pub struct UserId(pub i64);

impl UserId {
    #[cfg(test)]
    pub const USER1: UserId = UserId(1);
}


#[async_trait::async_trait]
impl FromRequestParts<AppState> for UserId {
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| ServerError::RequiredSessionToken)?;
        let session_token = Uuid::from_str(bearer.token()).map_err(|_| ServerError::InvalidSessionToken)?;
        state.pool.select_from_users(&SessionToken(session_token)).await
    }
}