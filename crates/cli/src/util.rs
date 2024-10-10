use std::path::PathBuf;
use std::process::Output;

pub const HTTP_SERVER_ADDR: &str = "https://gph-server.shuttleapp.rs";
pub const WS_SERVER_ADDR: &str = "wss://gph-server.shuttleapp.rs";

pub fn session_token_path() -> PathBuf {
    app_dir().join(".session")
}

pub fn app_dir() -> PathBuf {
    let dir = dirs_next::data_local_dir()
        .or_else(||dirs_next::data_dir())
        .expect("Failed to read data local or data directory");
    let gph = dir.join("gph");
    if !gph.exists() {
        std::fs::create_dir_all(&gph).expect("Failed to create app dir");
    }
    gph
}

pub fn colored_terminal_text(r: i32, g: i32, b: i32, text: &str) -> String {
    format!("\x1B[38;2;{};{};{}m{}\x1B[0m", r, g, b, text)
}

pub trait OutputErr {
    fn err_if_failed(self) -> std::io::Result<Output>;
}

impl OutputErr for Output {
    fn err_if_failed(self) -> std::io::Result<Output> {
        if self.status.success() {
            Ok(self)
        } else {
            Err(std::io::Error::other(String::from_utf8_lossy(&self.stderr)))
        }
    }
}