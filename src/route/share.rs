use crate::db;
use crate::db::rooms::RoomsTable;
use crate::error::ServerResult;
use crate::middleware::user_id::UserId;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{pin_mut, SinkExt, StreamExt};
use gph_core::types::GitResponse;
use sqlx::PgPool;


pub async fn share(
    user_id: UserId,
    State(pool): State<PgPool>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |ws| async move {
        let (mut ws_tx, mut ws_rx) = ws.split();
        tokio::select! {
             _ = listen_websocket(&mut ws_rx, &pool) => {},
            _ = listen_owner_channel(&mut ws_tx, pool.clone(), user_id) => {}
        };

        if let Err(e) = pool.update_room_status(user_id, false).await {
            tracing::error!("Failed to close room {e}");
        }

        if let Err(e) = ws_tx.close().await {
            tracing::error!("Failed close websocket({}): {e}", user_id.0);
        }
    })
}

async fn listen_owner_channel(
    ws: &mut SplitSink<WebSocket, Message>,
    pool: PgPool,
    user_id: UserId,
) -> ServerResult {
    let stream = db::channel::owner::listen(pool.clone(), user_id).await?;
    pin_mut!(stream);

    pool.update_room_status(user_id, true).await?;
    while let Some(git_request) = stream.next().await {
        // If return error, probably websocket has been closed.
        if ws.send(Message::Text(serde_json::to_string(&git_request).unwrap())).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

async fn listen_websocket(
    ws: &mut SplitStream<WebSocket>,
    pool: &PgPool,
) -> ServerResult {
    while let Some(Ok(message)) = ws.next().await {
        let Ok(message) = message.to_text() else {
            continue;
        };
        let Ok(git_response) = serde_json::from_str::<GitResponse>(message) else {
            continue;
        };
        db::channel::owner::response(pool, &git_response.id, &git_response.output).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::db::channel::guest::new_request;
    use crate::db::channel::RequestNotify;
    use crate::db::test::{DBInit, SESSION1};
    use crate::middleware::session_token::SessionToken;
    use crate::middleware::user_id::UserId;
    use crate::test::{start_server, TestResult};
    use futures_util::StreamExt;
    use gph_core::types::GitRequest;
    use reqwest::header;
    use sqlx::PgPool;
    use tokio::net::TcpStream;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::StatusCode;
    use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

    #[sqlx::test]
    async fn err_if_missing_session_token(pool: PgPool) {
        let port = start_server(pool).await;
        let error = connect_async(&format!("ws://localhost:{port}/share")).await.unwrap_err();
        if let tokio_tungstenite::tungstenite::Error::Http(response) = error {
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        } else {
            panic!("Invalid error type")
        }
    }

    #[sqlx::test]
    async fn err_if_invalid_user(pool: PgPool) {
        let port = start_server(pool).await;
        let status_code = connect_expect_err(port, &SessionToken::default()).await;
        assert_eq!(status_code, StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test]
    async fn ok_open(pool: PgPool) -> TestResult {
        pool.init().await;
        let port = start_server(pool).await;
        connect(port, &SESSION1).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn ok_recv_request(pool: PgPool) -> TestResult {
        pool.init().await;
        let port = start_server(pool.clone()).await;
        let mut ws = connect(port, &SESSION1).await?;
        let request_body = vec![1, 2, 3];
        let request_id = new_request(&pool, UserId::USER1, &request_body).await?;
        let request_notify = RequestNotify {
            to: UserId::USER1,
            id: request_id,
            path_info: "path".to_string(),
            request_method: "".to_string(),
            query_string: None,
            content_length: None,
            content_type: None,
        };
        db::channel::guest::request_to_owner(&pool, &request_notify).await?;
        let git_request = ws.next().await.unwrap()?;
        serde_json::from_str::<GitRequest>(git_request.to_text()?)?;
        Ok(())
    }

    async fn connect_expect_err(port: usize, session_token: &SessionToken) -> StatusCode {
        let error = connect(port, session_token)
            .await
            .unwrap_err();
        if let tokio_tungstenite::tungstenite::Error::Http(response) = error {
            response.status()
        } else {
            panic!("Invalid error type")
        }
    }

    async fn connect(port: usize, session_token: &SessionToken) -> tokio_tungstenite::tungstenite::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let mut request = format!("ws://localhost:{port}/share").into_client_request()?;
        request.headers_mut().insert(header::AUTHORIZATION, format!("Bearer {}", session_token.0).parse()?);
        connect_async(request)
            .await
            .map(|(ws, _)| ws)
    }
}