use axum::extract::FromRef;
use oauth2::{ClientId, ClientSecret};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;


#[derive(Clone)]
pub struct GithubCredentials {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

impl GithubCredentials {
    pub fn load(secret_store: SecretStore) -> Self {
        Self {
            client_id: ClientId::new(secret_store.get("CLIENT_ID").expect("Failed to read env CLIENT_ID")),
            client_secret: ClientSecret::new(secret_store.get("CLIENT_SECRET").expect("Failed to read env CLIENT_SECRET")),
        }
    }

    #[cfg(test)]
    pub fn load_test() -> GithubCredentials {
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