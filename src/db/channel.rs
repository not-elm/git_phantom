use crate::middleware::user_id::UserId;
use gph_core::types::{GitRequest, RequestId};

pub mod owner;
pub mod guest;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq)]
pub struct RequestNotify {
    pub to: UserId,
    pub id: RequestId,
    pub path_info: String,
    pub request_method: String,
    pub query_string: Option<String>,
    pub content_length: Option<String>,
    pub content_type: Option<String>,
}

pub fn convert_to_git_request(notify: RequestNotify, request_body: Vec<u8>) -> GitRequest {
    GitRequest {
        id: notify.id,
        path_info: notify.path_info,
        required_method: notify.request_method,
        query_string: notify.query_string,
        content_length: notify.content_length,
        content_type: notify.content_type,
        body: request_body,
    }
}

