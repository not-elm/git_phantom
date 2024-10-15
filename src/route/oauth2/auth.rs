use crate::state::GithubCredentials;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, CsrfToken, TokenUrl};


pub async fn auth(
    State(credential): State<GithubCredentials>
) -> Response {
    let (auth_url, _csrf_token) = oauth_client(credential.clone())
        .authorize_url(CsrfToken::new_random)
        .url();
    Redirect::to(auth_url.as_ref()).into_response()
}

fn oauth_client(credential: GithubCredentials) -> BasicClient {
    BasicClient::new(
        credential.client_id,
        Some(credential.client_secret),
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap()),
    )
}


#[cfg(test)]
mod tests {
    use crate::test::{auth_request, test_app, TestResult};
    use axum::http::StatusCode;
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn ok_redirect_status(pool: PgPool) -> TestResult {
        let app = test_app(pool).await;
        let response = app.oneshot(auth_request()).await?;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        Ok(())
    }
}