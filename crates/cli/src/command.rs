mod auth;
mod share;

use async_trait::async_trait;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum CliCommand {
    /// Signup using GitHub oauth2
    Auth(auth::Auth),

    ///  Share git repository
    Share(share::Share),
}

#[async_trait]
impl CommandExecutable for CliCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Auth(auth) => auth.execute().await,
            Self::Share(open) => open.execute().await,
        }
    }
}

#[async_trait]
pub trait CommandExecutable {
    async fn execute(self) -> anyhow::Result<()>;
}
