pub mod api;
pub mod db;

use axum::Router;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Latest frame of an in-flight agent session, published for live viewing.
pub struct LiveSession {
    pub game_id: String,
    pub strategy: String,
    pub frame_number: u64,
    pub max_frames: u64,
    pub rgba: Vec<u8>, // 426x240 RGBA8
}

/// Keyed by session_id. Uses a std Mutex because agent sessions update it
/// from a blocking thread.
pub type LiveMap = Arc<std::sync::Mutex<HashMap<String, LiveSession>>>;

pub struct AppState {
    pub db: db::Database,
    pub recordings_dir: String,
    pub live: LiveMap,
}

pub type SharedState = Arc<Mutex<AppState>>;

pub async fn run_server(port: u16, recordings_dir: String) -> Result<(), String> {
    std::fs::create_dir_all(&recordings_dir).map_err(|e| format!("recordings dir: {}", e))?;
    let db = db::Database::new(&format!("{}/aurex.db.json", recordings_dir))
        .map_err(|e| format!("db init: {}", e))?;

    let live: LiveMap = Arc::new(std::sync::Mutex::new(HashMap::new()));
    let state = Arc::new(Mutex::new(AppState { db, recordings_dir, live }));

    let app = Router::new()
        .merge(api::routes())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| format!("bind: {}", e))?;

    println!("Aurex server listening on http://0.0.0.0:{}", port);
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("serve: {}", e))?;

    Ok(())
}
