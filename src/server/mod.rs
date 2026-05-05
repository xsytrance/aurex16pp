pub mod api;
pub mod db;

use axum::Router;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub db: db::Database,
    pub recordings_dir: String,
}

pub type SharedState = Arc<Mutex<AppState>>;

pub async fn run_server(port: u16, recordings_dir: String) -> Result<(), String> {
    let db = db::Database::new(&format!("{}/aurex.db", recordings_dir))
        .map_err(|e| format!("db init: {}", e))?;

    let state = Arc::new(Mutex::new(AppState { db, recordings_dir }));

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
