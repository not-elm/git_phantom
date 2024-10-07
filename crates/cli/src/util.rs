use std::path::PathBuf;

pub const HTTP_SERVER_ADDR: &str = "http://localhost:8000";


pub fn session_token_path() -> PathBuf {
    app_dir().join(".session")
}

fn app_dir() -> PathBuf {
    let dir = dirs_next::data_local_dir().unwrap();
    let gph = dir.join("gph");
    if !gph.exists() {
        std::fs::create_dir_all(&gph).expect("Failed to create app dir");
    }
    gph
}