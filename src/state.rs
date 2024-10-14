use axum::extract::FromRef;
use oauth2::{ClientId, ClientSecret};
use sqlx::PgPool;

#[derive(Clone)]
pub struct GithubCredentials {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

impl GithubCredentials {
    pub fn load() -> GithubCredentials {
        use std::env;
        GithubCredentials {
            client_id: ClientId::new(env::var("CLIENT_ID").unwrap()),
            client_secret: ClientSecret::new(env::var("CLIENT_SECRET").unwrap()),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub github_credentials: GithubCredentials,
}

impl FromRef<AppState> for PgPool {
    #[inline]
    fn from_ref(input: &AppState) -> Self {
        input.pool.clone()
    }
}

impl FromRef<AppState> for GithubCredentials {
    #[inline]
    fn from_ref(input: &AppState) -> Self {
        input.github_credentials.clone()
    }
}