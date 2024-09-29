mod route;

use axum::{routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = app();
    Ok(router.into())
}

fn app() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .nest("/room", room_router())
}

fn room_router() -> Router {
    Router::new()
        .route("/open", get(route::room::open))
}