use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i64);

impl UserId {
    #[cfg(test)]
    pub const USER1: UserId = UserId(1);
}
