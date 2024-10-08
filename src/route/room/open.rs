use crate::db;
use crate::error::ServerResult;
use crate::middleware::user_id::UserId;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::stream::SplitSink;
use futures_util::{pin_mut, SinkExt, StreamExt};
use sqlx::PgPool;

pub async fn open(
    user_id: UserId,
    State(pool): State<PgPool>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|mut ws| async move {
        let (mut ws_tx, ws_rx) = ws.split();
        listen_owner_channel(&mut ws_tx, pool, user_id).await;
    })
}

async fn listen_owner_channel(
    ws: &mut SplitSink<WebSocket, Message>,
    pool: PgPool,
    user_id: UserId,
) -> ServerResult {
    let stream = db::channel::owner::listen(pool, user_id).await?;
    pin_mut!(stream);

    while let Some(git_request) = stream.next().await {
        // If return error, probably websocket has been closed.
        if ws.send(Message::Text(serde_json::to_string(&git_request).unwrap())).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}
