use serde::{Deserialize, Serialize};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

#[derive(Debug, Serialize, Deserialize)]
pub enum UserMessage {
    Sdp{
        offer_description: RTCSessionDescription
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OwnerMessage {
    Sdp{
        answer_description: RTCSessionDescription
    }
}

