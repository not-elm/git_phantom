use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;

pub async fn open(
    ws: WebSocketUpgrade
) -> impl IntoResponse {
    ws.on_upgrade(|mut ws| async move {
        while let Some(Ok(_)) = ws.recv().await {}
    })
}

