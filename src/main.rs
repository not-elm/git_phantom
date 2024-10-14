mod route;
mod db;
mod middleware;
mod error;
mod state;

use crate::state::{AppState, GithubCredentials};
use axum::routing::put;
use axum::{routing::get, Router};
use sqlx::PgPool;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&db_url).await?;
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrate");
    let app = app(AppState {
        pool,
        github_credentials: GithubCredentials::load(),
    });
    #[cfg(debug_assertions)]
    http::start_server(app).await?;
    #[cfg(not(debug_assertions))]
    https::start_server(app).await?;
    Ok(())
}

#[cfg(debug_assertions)]
mod http {
    use axum::Router;
    use tokio::net::TcpListener;

    pub async fn start_server(app: Router) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("0.0.0.0:8080").await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[cfg(not(debug_assertions))]
mod https {
    use axum::Router;
    use axum_server::tls_rustls::RustlsConfig;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    pub async fn start_server(app: Router) -> Result<(), Box<dyn std::error::Error>> {
        let config = RustlsConfig::from_pem_file(
            PathBuf::from(std::env::var("CERT_PEM").expect("Failed to read CERT_PEM")),
            PathBuf::from(std::env::var("KEY_PEM").expect("Failed to read KEY_PEM")),
        )
            .await?;

        let addr = SocketAddr::from(([0, 0, 0, 0], 443));
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await?;
        Ok(())
    }
}


fn app(app_state: AppState) -> Router {
    Router::new()
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
            github_credentials: GithubCredentials::load(),
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