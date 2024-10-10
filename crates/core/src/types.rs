use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct RequestId(pub Uuid);

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq)]
pub struct GitRequest {
    pub id: RequestId,
    pub path_info: String,
    pub required_method: String,
    pub query_string: Option<String>,
    pub content_length: Option<String>,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GitResponse {
    pub id: RequestId,
    pub output: Vec<u8>,
}
