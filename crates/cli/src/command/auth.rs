use crate::command::CommandExecutable;
use crate::util::{session_token_path, HTTP_SERVER_ADDR};
use async_trait::async_trait;
use axum::extract::{Query, State};
use axum::Router;
use clap::Args;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;

#[derive(Args, Debug, Clone)]
pub struct Auth {
    /// Port of the local server that will receive the auth-code.
    /// (Default is 7740) 
    #[clap(short, long, default_value = "7740")]
    listen_port: u64,
}

#[async_trait]
impl CommandExecutable for Auth {
    async fn execute(self) -> anyhow::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1);
        start_http_server(self.listen_port, tx);
        webbrowser::open(&format!("{HTTP_SERVER_ADDR}/oauth2/auth"))?;

        let auth_code = rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to recv auth code"))?;
        let session_token = register_user(auth_code).await?;
        std::fs::write(session_token_path(), session_token)?;
        println!("{}", colored(255, 255, 0, "Success!"));
        Ok(())
    }
}

fn colored(r: i32, g: i32, b: i32, text: &str) -> String {
    format!("\x1B[38;2;{};{};{}m{}\x1B[0m", r, g, b, text)
}

fn start_http_server(port: u64, tx: Sender<String>) {
    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .expect("Failed to create tcp listener");
        let router = Router::new()
            .route("/oauth2/callback", axum::routing::get(oauth2_callback))
            .with_state(tx);
        axum::serve(listener, router)
            .await
            .expect("Failed to start http server");
    });
}

async fn oauth2_callback(
    Query(mut query): Query<HashMap<String, String>>,
    State(tx): State<Sender<String>>,
) {
    if let Some(code) = query.remove("code") {
        tx.send(code).await.expect("Failed to pass auth code");
    }
}

async fn register_user(auth_code: String) -> anyhow::Result<String> {
    let session_token = reqwest::Client::new()
        .put(format!(
            "{HTTP_SERVER_ADDR}/oauth2/register?code={auth_code}"
        ))
        .send()
        .await?
        .text()
        .await?;
    Ok(session_token)
}
