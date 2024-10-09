mod auth;
mod open;

use async_trait::async_trait;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum CliCommand {
    Auth(auth::Auth),
    Open(open::Open),
}

#[async_trait]
impl CommandExecutable for CliCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Auth(auth) => auth.execute().await,
            Self::Open(open) => open.execute().await,
        }
    }
}

#[async_trait]
pub trait CommandExecutable {
    async fn execute(self) -> anyhow::Result<()>;
}
