use crate::command::{CliCommand, CommandExecutable};
use clap::Parser;

mod command;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    CliCommand::parse().execute().await
}
