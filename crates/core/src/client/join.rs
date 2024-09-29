use crate::client::{create_peer, WsSender};
use crate::types::UserMessage;
use futures_util::{SinkExt, StreamExt};
use webrtc::peer_connection::RTCPeerConnection;

pub async fn join() -> anyhow::Result<()> {
    let (ws, _) = tokio_tungstenite::connect_async("ws://localhost:8000/room/open").await?;
    let (mut tx, rx) = ws.split();

    let mut peer = create_peer().await?;
    peer.on_ice_candidate(Box::new(|_| {
        Box::pin(async move {})
    }));
    send_offer(&mut peer, &mut tx).await?;
    Ok(())
}

async fn send_offer(
    peer: &mut RTCPeerConnection,
    ws: &mut WsSender,
) -> anyhow::Result<()> {
    let offer_description = peer.create_offer(None).await?;
    peer.set_local_description(offer_description.clone()).await?;
    let message = serde_json::to_string(&UserMessage::Sdp { offer_description })?;
    ws.send(message.into()).await?;
    Ok(())
}