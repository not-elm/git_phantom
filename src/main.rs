mod route;
mod db;
mod middleware;
mod error;
mod state;

use crate::state::{AppState, GithubCredentials};
use axum::routing::put;
use axum::{routing::get, Router};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] store: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrate");
    let router = app(AppState {
        pool,
        github_credentials: GithubCredentials::load(store),
    });
    Ok(router.into())
}

fn app(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async move {
            "hello world"
        }))
        .nest("/oauth2", oauth2_router())
        .nest("/room", room_router())
        .with_state(app_state)
}

fn oauth2_router() -> Router<AppState> {
    Router::new()
        .route("/auth", get(route::oauth2::auth))
        .route("/register", put(route::oauth2::register))
}

fn room_router() -> Router<AppState> {
    Router::new()
        .route("/open", get(route::room::open))
}


#[cfg(test)]
pub(crate) mod test {
    use crate::app;
    use crate::state::{AppState, GithubCredentials};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::Router;
    use sqlx::PgPool;

    pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    pub async fn test_app(pool: PgPool) -> Router {
        app(AppState {
            pool,
            github_credentials: GithubCredentials::load_test(),
        })
    }

    pub fn auth_request() -> Request {
        Request::get("/oauth2/auth").body(Body::empty()).unwrap()
    }
}