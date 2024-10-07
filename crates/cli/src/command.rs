mod auth;

use async_trait::async_trait;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum CliCommand {
    Auth(auth::Auth)
}

#[async_trait]
impl CommandExecutable for CliCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Auth(auth) => auth.execute().await
        }
    }
}


#[async_trait]
pub trait CommandExecutable {
    async fn execute(self) -> anyhow::Result<()>;
}