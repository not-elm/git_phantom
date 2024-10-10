use crate::command::CommandExecutable;
use crate::util::{app_dir, colored_terminal_text, session_token_path, OutputErr, HTTP_SERVER_ADDR, WS_SERVER_ADDR};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::Args;
use futures_util::{SinkExt, StreamExt};
use gph_core::types::{GitRequest, GitResponse};
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[derive(Debug, Clone, Args)]
pub struct Share {
    /// Remote repository name.
    #[clap(short, long)]
    pub repository: Option<String>,
}

#[async_trait]
impl CommandExecutable for Share {
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
        let repository_name = change_repository_extension(repository_name);
        let git_remote_url = create_git_remote_url(&session_token, &repository_name).await?;

        let _ = std::fs::remove_dir_all(git_root()?.join(&repository_name));
        git_init(&repository_name).await?;
        let result = execute_share(&session_token, &git_remote_url, &repository_name).await;

        if let Err(e) = git_remote_remove().await {
            eprintln!("{e}");
        }
        if let Err(e) = std::fs::remove_dir_all(git_root()?.join(repository_name)) {
            eprintln!("{e}");
        }
        result?;

        Ok(())
    }
}

async fn execute_share(session_token: &str, git_remote_url: &str, repository_name: &str) -> anyhow::Result<()> {
    let _ = git_remote_remove().await;
    git_add_remote(repository_name).await?;
    git_push_all().await?;
    git_set_remote(git_remote_url).await?;
    git_set_http_receive_pack(repository_name).await?;

    let mut request = format!("{WS_SERVER_ADDR}/share").into_client_request()?;
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {session_token}").parse()?);
    let (ws, _) = tokio_tungstenite::connect_async(request).await?;

    let mut clipboard = Clipboard::new()?;
    if let Err(e) = clipboard.set_text(git_remote_url) {
        eprintln!("{e}");
    }

    println!("{} {git_remote_url}", colored_terminal_text(255, 255, 0, "Git remote url:"));
    tokio::select! {
            result= tokio::spawn(async move{ websocket_handle(ws).await }) => result??,
            result = spawn_shell() => result?
    }
    Ok(())
}

fn change_repository_extension(repository: String) -> String {
    if repository.ends_with(".git") {
        repository
    } else {
        format!("{repository}.git")
    }
}

async fn websocket_handle(
    mut ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> anyhow::Result<()> {
    while let Some(Ok(message)) = ws.next().await {
        let Ok(message) = message.to_text() else {
            continue;
        };
        let Ok(git_request) = serde_json::from_str::<GitRequest>(message) else {
            continue;
        };
        let request_id = git_request.id;
        let output = execute_git_http_backend(git_request).await?;
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

async fn create_git_remote_url(session_token: &str, repository_name: &str) -> reqwest::Result<String> {
    let user_id = reqwest::Client::new()
        .get(format!("{HTTP_SERVER_ADDR}/user_id"))
        .bearer_auth(session_token)
        .send()
        .await?
        .text()
        .await?;
    Ok(format!("{HTTP_SERVER_ADDR}/git/{user_id}/{repository_name}"))
}

async fn git_init(repository: &str) -> std::io::Result<()> {
    Command::new("git")
        .arg("init")
        .arg("--shared")
        .arg("--bare")
        .arg("--initial-branch").arg("main")
        .arg(git_root()?.join(repository))
        .output()
        .await?
        .err_if_failed()?;
    Ok(())
}

async fn git_set_http_receive_pack(repository: &str) -> std::io::Result<()> {
    Command::new("git")
        .arg("config")
        .arg("http.receivepack")
        .arg("true")
        .current_dir(git_root()?.join(repository))
        .output()
        .await?
        .err_if_failed()?;
    Ok(())
}

async fn git_add_remote(repository: &str) -> std::io::Result<()> {
    Command::new("git")
        .arg("remote")
        .arg("add")
        .arg("gph")
        .arg(git_root()?.join(repository))
        .output()
        .await?
        .err_if_failed()?;
    Ok(())
}

async fn git_push_all() -> std::io::Result<()> {
    Command::new("git")
        .arg("push")
        .arg("gph")
        .arg("--all")
        .output()
        .await?;
    Ok(())
}

async fn git_set_remote(git_remote_url: &str) -> std::io::Result<()> {
    Command::new("git")
        .arg("remote")
        .arg("set-url")
        .arg("gph")
        .arg(git_remote_url)
        .output()
        .await?
        .err_if_failed()?;
    Ok(())
}

async fn git_remote_remove() -> std::io::Result<()> {
    Command::new("git")
        .arg("remote")
        .arg("rm")
        .arg("gph")
        .output()
        .await?
        .err_if_failed()?;
    Ok(())
}

async fn execute_git_http_backend(request: GitRequest) -> std::io::Result<Vec<u8>> {
    let mut cmd = Command::new("git");
    cmd.arg("http-backend");

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
            format!("/{}", request.path_info),
        )
        .env("REQUEST_METHOD", request.required_method)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if !request.body.is_empty() {
        http_backend
            .stdin
            .as_mut()
            .unwrap()
            .write_all(request.body.as_ref())
            .await?;
    }

    Ok(http_backend.wait_with_output().await?.err_if_failed()?.stdout)
}

async fn spawn_shell() -> anyhow::Result<()> {
    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("powershell.exe")
    } else {
        Command::new("sh")
    };
    cmd.spawn()?.wait_with_output().await?;
    Ok(())
}

fn git_root() -> std::io::Result<PathBuf> {
    let dir = app_dir().join("git");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create git root dir");
    }
    Ok(dir)
}
