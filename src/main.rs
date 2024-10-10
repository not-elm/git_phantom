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
        .route("/user_id", get(route::user_id))
        .route("/share", get(route::share))
        .route("/git/:user_id/*path", get(route::git).post(route::git))
        .with_state(app_state)
}

fn oauth2_router() -> Router<AppState> {
    Router::new()
        .route("/auth", get(route::oauth2::auth))
        .route("/register", put(route::oauth2::register))
}


#[cfg(test)]
pub(crate) mod test {
    use crate::app;
    use crate::state::{AppState, GithubCredentials};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::Router;
    use sqlx::PgPool;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::net::TcpListener;

    pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    pub static PORT: AtomicUsize = AtomicUsize::new(5100);

    pub async fn test_app(pool: PgPool) -> Router {
        app(AppState {
            pool,
            github_credentials: GithubCredentials::load_test(),
        })
    }

    pub fn auth_request() -> Request {
        Request::get("/oauth2/auth").body(Body::empty()).unwrap()
    }

    pub async fn start_server(pool: PgPool) -> usize {
        let port = PORT.fetch_add(1, Ordering::Relaxed);
        tokio::spawn(async move {
            let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
            let app = test_app(pool).await;
            axum::serve(listener, app).await.unwrap();
        });
        port
    }
}