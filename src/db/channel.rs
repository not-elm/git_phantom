use crate::middleware::user_id::UserId;
use gph_core::types::RequestId;
use sqlx::{Executor, Row};

pub mod owner;
pub mod guest;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq)]
pub struct RequestNotify {
    pub to: UserId,
    pub id: RequestId,
    pub path_info: String,
    pub required_method: String,
    pub query_string: Option<String>,
    pub content_length: Option<String>,
    pub content_type: Option<String>,
}


