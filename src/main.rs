mod route;

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
    let router = app();
    Ok(router.into())
}

fn app() -> Router {
    Router::new()
        .route("/", get(|| async move {
            "hello world"
        }))
        .nest("/room", room_router())
}

fn room_router() -> Router {
    Router::new()
        .route("/open", get(route::room::open))
}