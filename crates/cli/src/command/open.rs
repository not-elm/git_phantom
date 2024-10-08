use crate::command::CommandExecutable;
use crate::util::{app_dir, session_token_path, WS_SERVER_ADDR};
use async_trait::async_trait;
use clap::Args;
use futures_util::{SinkExt, StreamExt};
use gph_core::types::{GitRequest, GitResponse};
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[derive(Debug, Clone, Args)]
pub struct Open {
    #[clap(short, long)]
    pub repository: Option<String>,
}

#[async_trait]
impl CommandExecutable for Open {
    async fn execute(self) -> anyhow::Result<()> {
        let session_token = std::fs::read_to_string(session_token_path())?;
        let repository_name = match self.repository {
            Some(repository) => repository,
            None => env::current_dir()?
                .file_name()
                .and_then(|f| f.to_str())
                .map(String::from)
                .expect("Failed to read current dir name"),
        };
        git_init(&repository_name).await?;

        let mut request = format!("{WS_SERVER_ADDR}/room/open").into_client_request()?;
        request
            .headers_mut()
            .insert("Authorization", format!("Bearer {session_token}").parse()?);
        let (ws, _) = tokio_tungstenite::connect_async(request).await?;

        tokio::select! {
            _ = websocket_handle(ws, &repository_name) => {},
            _ = tokio::signal::ctrl_c() => {}
        }

        std::fs::remove_dir_all(git_root()?.join(format!("{repository_name}.git")))?;
        Ok(())
    }
}

async fn websocket_handle(
    mut ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    repository: &str,
) -> anyhow::Result<()> {
    while let Some(Ok(message)) = ws.next().await {
        let Ok(message) = message.to_text() else {
            continue;
        };
        let Ok(git_request) = serde_json::from_str::<GitRequest>(message) else {
            continue;
        };
        let request_id = git_request.id.clone();
        let output = execute_git_http_backend(git_request, repository)?;
        ws.send(Message::Text(
            serde_json::to_string(&GitResponse {
                id: request_id,
                output,
            })
            .unwrap(),
        ))
        .await?;
    }
    Ok(())
}

async fn git_init(repository: &str) -> std::io::Result<()> {
    Command::new("git")
        .arg("init")
        .arg("--shared")
        .arg("--bare")
        .arg(git_root()?.join(repository))
        .output()?;
    Ok(())
}

fn execute_git_http_backend(request: GitRequest, repository: &str) -> std::io::Result<Vec<u8>> {
    let mut cmd = Command::new("/usr/lib/git-core/git-http-backend");
    if let Some(query) = request.query_string {
        cmd.env("QUERY_STRING", query);
    }
    if let Some(content_length) = request.content_length {
        cmd.env("CONTENT_LENGTH", content_length);
    }
    if let Some(content_type) = request.content_type {
        cmd.env("CONTENT_TYPE", content_type);
    }

    let mut http_backend = cmd
        .env("GIT_PROJECT_ROOT", git_root()?)
        .env("GIT_HTTP_EXPORT_ALL", "1")
        .env(
            "PATH_INFO",
            format!("/{repository}.git/{}", request.path_info),
        )
        .env("REQUEST_METHOD", request.required_method)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    if !request.body.is_empty() {
        http_backend
            .stdin
            .as_mut()
            .unwrap()
            .write_all(request.body.as_ref())?;
    }

    Ok(http_backend.wait_with_output()?.stdout)
}

fn git_root() -> std::io::Result<PathBuf> {
    let dir = app_dir().join("git");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create git root dir");
    }
    Ok(dir)
}
