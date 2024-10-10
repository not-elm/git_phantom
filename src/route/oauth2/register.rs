use crate::db::users::UsersTable;
use crate::error::{ServerError, ServerResult};
use crate::middleware::user_id::UserId;
use crate::state::GithubCredentials;
use axum::extract::{Query, State};
use axum::http::header;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;


pub async fn register(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<PgPool>,
    State(github_credentials): State<GithubCredentials>,
) -> ServerResult<String> {
    let Some(auth_code) = query.get("code") else {
        return Err(ServerError::MissingAuthCode);
    };
    let access_token = fetch_access_token(&github_credentials, auth_code)
        .await
        .map_err(|_| ServerError::FailedConnectGithubApi)?
        .access_token;
    let user_id = fetch_github_id(&access_token).await?;
    let session_token = pool.insert_into_users(&user_id).await?;
    Ok(session_token.to_string())
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

async fn fetch_access_token(credential: &GithubCredentials, code: &str) -> reqwest::Result<AccessTokenResponse> {
    reqwest::Client::new()
        .post("https://github.com/login/oauth/access_token")
        .header(header::ACCEPT, "application/json")
        .json(&serde_json::json!({
            "client_id": credential.client_id,
            "client_secret": credential.client_secret,
            "code": code,
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

async fn fetch_github_id(access_token: &str) -> ServerResult<UserId> {
    let json = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header(header::USER_AGENT, "meltos_app")
        .header(header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(access_token)
        .send()
        .await
        .and_then(|response| response.error_for_status())
        .map_err(|_| ServerError::FailedConnectGithubApi)?
        .json::<HashMap<String, serde_json::Value>>()
        .await
        .map_err(|_| ServerError::FailedConnectGithubApi)?;
    let user_id = json
        .get("id")
        .ok_or_else(|| ServerError::FailedConnectGithubApi)?
        .as_i64()
        .ok_or_else(|| ServerError::FailedConnectGithubApi)?;
    Ok(UserId(user_id))
}

#[cfg(test)]
mod tests {
    use crate::test::{test_app, TestResult};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn err_if_code_is_not_set(pool: PgPool) -> TestResult {
        let app = test_app(pool).await;
        let response = app.oneshot(Request::put("/oauth2/register").body(Body::empty()).unwrap()).await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }
}