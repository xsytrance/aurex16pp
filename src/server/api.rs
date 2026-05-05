use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};
use axum::body::Body;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::server::db::Recording;
use crate::server::SharedState;

#[derive(Deserialize)]
struct CreateGameRequest {
    title: String,
    genre: String,
    description: String,
}

#[derive(Serialize)]
struct CreateGameResponse {
    id: String,
    title: String,
    status: String,
    created_at: String,
}

#[derive(Serialize)]
struct GameSummary {
    id: String,
    title: String,
    genre: String,
    status: String,
    created_at: String,
    recording_count: usize,
}

#[derive(Serialize)]
struct GameListResponse {
    games: Vec<GameSummary>,
}

#[derive(Serialize)]
struct GameDetailResponse {
    id: String,
    title: String,
    genre: String,
    description: String,
    status: String,
    created_at: String,
    recordings: Vec<Recording>,
}

#[derive(Deserialize)]
struct PlayRequest {
    strategy: String,
    max_frames: u64,
}

#[derive(Serialize)]
struct PlayResponse {
    session_id: String,
    status: &'static str,
    recording_path: String,
}

#[derive(Serialize)]
struct RecordingListResponse {
    recordings: Vec<Recording>,
}

#[derive(Serialize)]
struct StrategiesResponse {
    strategies: Vec<&'static str>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn routes() -> Router<SharedState> {
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/games/create", post(create_game))
        .route("/api/games", get(list_games))
        .route("/api/games/:id", get(get_game))
        .route("/api/games/:id/play", post(play_game))
        .route("/api/recordings", get(list_recordings))
        .route("/api/recordings/:id", get(stream_recording))
        .route("/api/recordings/:id/info", get(get_recording_info))
        .route("/api/strategies", get(list_strategies));

    let static_files = Router::new().nest_service(
        "/",
        ServeDir::new("webapp/dist").fallback(ServeFile::new("webapp/dist/index.html")),
    );

    api.merge(static_files)
        .layer(CorsLayer::permissive())
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

async fn create_game(
    State(state): State<SharedState>,
    Json(body): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, StatusCode> {
    let state = state.lock().await;
    let game = state
        .db
        .create_game(&body.title, &body.genre, &body.description)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateGameResponse {
        id: game.id,
        title: game.title,
        status: game.status,
        created_at: game.created_at.to_rfc3339(),
    }))
}

async fn list_games(
    State(state): State<SharedState>,
) -> Result<Json<GameListResponse>, StatusCode> {
    let state = state.lock().await;
    let games = state.db.list_games();
    let recordings = state.db.list_recordings();

    let summaries: Vec<GameSummary> = games
        .into_iter()
        .map(|g| {
            let recording_count = recordings
                .iter()
                .filter(|r| r.game_id == g.id)
                .count();
            GameSummary {
                id: g.id,
                title: g.title,
                genre: g.genre,
                status: g.status,
                created_at: g.created_at.to_rfc3339(),
                recording_count,
            }
        })
        .collect();

    Ok(Json(GameListResponse { games: summaries }))
}

async fn get_game(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<GameDetailResponse>, StatusCode> {
    let state = state.lock().await;
    let game = state.db.get_game(&id).ok_or(StatusCode::NOT_FOUND)?;
    let recordings = state.db.list_recordings_for_game(&id);

    Ok(Json(GameDetailResponse {
        id: game.id,
        title: game.title,
        genre: game.genre,
        description: game.description,
        status: game.status,
        created_at: game.created_at.to_rfc3339(),
        recordings,
    }))
}

async fn play_game(
    State(shared_state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<PlayRequest>,
) -> Result<Json<PlayResponse>, StatusCode> {
    let guard = shared_state.lock().await;
    let game = guard.db.get_game(&id).ok_or(StatusCode::NOT_FOUND)?;
    let recordings_dir = guard.recordings_dir.clone();
    drop(guard);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let session_id = format!("{}_{}", game.id, timestamp);
    let recording_path = format!("{}/session_{}_{}.mp4", recordings_dir, game.id, timestamp);

    // Update game status to generating
    {
        let guard = shared_state.lock().await;
        let _ = guard.db.update_game_status(&id, "generating");
    }

    let strategy_name = body.strategy.clone();
    let max_frames = body.max_frames;
    let game_id = id.clone();
    let state_clone = Arc::clone(&shared_state);
    let session_id_clone = session_id.clone();
    let recording_path_clone = recording_path.clone();

    tokio::task::spawn_blocking(move || {
        use crate::agent_session::{AgentSession, strategy_by_name};

        let strategy = strategy_by_name(&strategy_name);
        let mut session = AgentSession::new(&game_id, strategy, true, &recordings_dir)
            .map_err(|e| format!("create session: {}", e))?;
        let result = session.run_for_frames(max_frames)
            .map_err(|e| format!("run frames: {}", e))?;

        let file_size = std::fs::metadata(&recording_path_clone)
            .map(|m| m.len())
            .unwrap_or(0);

        let recording = Recording {
            id: session_id_clone,
            game_id: game_id.clone(),
            path: recording_path_clone,
            strategy: strategy_name,
            frames: result.frames_played,
            duration_secs: result.frames_played as f64 / 60.0,
            file_size,
            created_at: chrono::Utc::now(),
        };

        let guard = state_clone.blocking_lock();
        let _ = guard.db.add_recording(recording);
        let _ = guard.db.update_game_status(&game_id, "ready");

        Ok::<_, String>(())
    });

    Ok(Json(PlayResponse {
        session_id,
        status: "started",
        recording_path,
    }))
}

async fn list_recordings(
    State(state): State<SharedState>,
) -> Result<Json<RecordingListResponse>, StatusCode> {
    let state = state.lock().await;
    let recordings = state.db.list_recordings();
    Ok(Json(RecordingListResponse { recordings }))
}

async fn stream_recording(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Response, StatusCode> {
    let state = state.lock().await;
    let recording = state.db.get_recording(&id).ok_or(StatusCode::NOT_FOUND)?;
    let path = recording.path.clone();
    drop(state);

    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .body(body)
        .unwrap())
}

async fn get_recording_info(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Recording>, StatusCode> {
    let state = state.lock().await;
    let recording = state.db.get_recording(&id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(recording))
}

async fn list_strategies() -> impl IntoResponse {
    Json(StrategiesResponse {
        strategies: vec!["explorer", "passive", "aggressive"],
    })
}
