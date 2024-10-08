use crate::middleware::user_id::UserId;
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;

pub async fn open(
    user_id: UserId,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|mut ws| async move {
        while let Some(Ok(message)) = ws.recv().await {}
    })
}

