use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionToken(pub Uuid);

impl Display for SessionToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl SessionToken {
    #[cfg(test)]
    pub(crate) fn max() -> Self {
        Self(Uuid::max())
    }
}