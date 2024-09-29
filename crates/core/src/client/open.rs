use crate::client::{create_peer, WsReceiver, WsSender};
use crate::types::UserMessage;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

pub async fn open() -> anyhow::Result<()> {
    let (ws, _) = tokio_tungstenite::connect_async("ws://localhost:8000/room/open").await?;
    let (tx, rx) = ws.split();

    let mut peer = create_peer().await?;
    let mut data_channel = peer.create_data_channel("data", None).await?;
    data_channel.on_open(Box::new(move || {
        println!("data channel open");
        Box::pin(async move {})
    }));
    websocket_handle(&mut peer, tx, rx).await?;
    Ok(())
}

async fn websocket_handle(
    peer: &mut RTCPeerConnection,
    mut tx: WsSender,
    mut rx: WsReceiver,
) -> anyhow::Result<()> {
    while let Some(Ok(message)) = rx.next().await {
        let Message::Text(message) = message else {
            continue;
        };

        let Ok(message) = serde_json::from_str::<UserMessage>(&message) else {
            continue;
        };
        match message {
            UserMessage::Sdp { offer_description } => send_answer(&mut tx, peer, offer_description).await?,
        }
    }
    Ok(())
}

async fn send_answer(
    ws: &mut WsSender,
    peer: &mut RTCPeerConnection,
    offer_description: RTCSessionDescription,
) -> anyhow::Result<()> {
    peer.set_remote_description(offer_description).await?;
    let answer = peer.create_answer(None).await?;
    ws.send(serde_json::to_string(&answer).unwrap().into()).await?;
    peer.set_local_description(answer).await?;
    Ok(())
}
