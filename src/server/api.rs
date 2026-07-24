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
use crate::server::{LiveSession, SharedState};

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
    #[serde(default)]
    audio_profile: Option<String>,
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
        .route("/api/strategies", get(list_strategies))
        .route("/api/live", get(list_live))
        .route("/api/live/:id/frame", get(live_frame))
        .route("/api/recordings/:id/replay", post(replay_recording));

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
    let live = Arc::clone(&guard.live);
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
    let audio_profile = body
        .audio_profile
        .as_deref()
        .and_then(crate::aurex::runtime::MixProfile::parse)
        .unwrap_or(crate::aurex::runtime::MixProfile::Default);
    let game_id = id.clone();
    let state_clone = Arc::clone(&shared_state);
    let session_id_clone = session_id.clone();
    let recording_path_clone = recording_path.clone();

    // Register the live session so viewers can watch while the agent plays.
    live.lock().unwrap().insert(
        session_id.clone(),
        LiveSession {
            game_id: game_id.clone(),
            strategy: strategy_name.clone(),
            frame_number: 0,
            max_frames,
            rgba: vec![0; 426 * 240 * 4],
        },
    );

    tokio::task::spawn_blocking(move || {
        use crate::agent_session::{AgentSession, FrameObserver, strategy_by_name};

        let live_for_obs = Arc::clone(&live);
        let obs_session_id = session_id_clone.clone();
        let observer: FrameObserver = Box::new(move |frame, fb| {
            // Publish every 3rd frame (~20 FPS) — plenty for a live preview.
            if frame % 3 != 0 {
                return;
            }
            let mut rgba = Vec::with_capacity(fb.len() * 4);
            for &pixel in fb {
                let r5 = ((pixel >> 10) & 0x1F) as u8;
                let g5 = ((pixel >> 5) & 0x1F) as u8;
                let b5 = (pixel & 0x1F) as u8;
                rgba.push((r5 << 3) | (r5 >> 2));
                rgba.push((g5 << 3) | (g5 >> 2));
                rgba.push((b5 << 3) | (b5 >> 2));
                rgba.push(255);
            }
            if let Ok(mut map) = live_for_obs.lock() {
                if let Some(entry) = map.get_mut(&obs_session_id) {
                    entry.frame_number = frame;
                    entry.rgba = rgba;
                }
            }
        });

        let outcome = (|| {
            let strategy = strategy_by_name(&strategy_name);
            let mut session =
                AgentSession::new(&game_id, strategy, true, &recordings_dir, audio_profile)
                    .map_err(|e| format!("create session: {}", e))?;
            session
                .run_session(max_frames, true, Some(observer))
                .map_err(|e| format!("run frames: {}", e))
        })();

        live.lock().unwrap().remove(&session_id_clone);

        let guard = state_clone.blocking_lock();
        match outcome {
            Ok(result) => {
                let actual_path = result.recording_path.unwrap_or(recording_path_clone);
                let file_size = std::fs::metadata(&actual_path).map(|m| m.len()).unwrap_or(0);

                let recording = Recording {
                    id: session_id_clone,
                    game_id: game_id.clone(),
                    path: actual_path,
                    strategy: strategy_name,
                    frames: result.frames_played,
                    duration_secs: result.frames_played as f64 / 60.0,
                    file_size,
                    created_at: chrono::Utc::now(),
                };

                let _ = guard.db.add_recording(recording);
                let _ = guard.db.update_game_status(&game_id, "ready");
            }
            Err(e) => {
                eprintln!("Agent session {} failed: {}", session_id_clone, e);
                let _ = guard.db.update_game_status(&game_id, "failed");
            }
        }
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
    headers: axum::http::HeaderMap,
) -> Result<Response, StatusCode> {
    let state = state.lock().await;
    let recording = state.db.get_recording(&id).ok_or(StatusCode::NOT_FOUND)?;
    let path = recording.path.clone();
    drop(state);

    let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let total = file
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    // Honor "bytes=start-[end]" so the <video> player can seek.
    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("bytes="))
        .and_then(|spec| {
            let (start_s, end_s) = spec.split_once('-')?;
            let start: u64 = start_s.parse().ok()?;
            let end: u64 = if end_s.is_empty() {
                total.saturating_sub(1)
            } else {
                end_s.parse().ok()?
            };
            (start <= end && end < total).then_some((start, end))
        });

    let (start, end) = match range {
        Some(r) => r,
        None => (0, total.saturating_sub(1)),
    };
    let len = end - start + 1;

    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    file.seek(std::io::SeekFrom::Start(start))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let stream = tokio_util::io::ReaderStream::new(file.take(len));
    let body = Body::from_stream(stream);

    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, len);

    if range.is_some() {
        builder = builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, total));
    } else {
        builder = builder.status(StatusCode::OK);
    }

    Ok(builder.body(body).unwrap())
}

#[derive(Serialize)]
struct LiveSessionSummary {
    session_id: String,
    game_id: String,
    strategy: String,
    frame_number: u64,
    max_frames: u64,
}

#[derive(Serialize)]
struct LiveListResponse {
    sessions: Vec<LiveSessionSummary>,
}

async fn list_live(State(state): State<SharedState>) -> Json<LiveListResponse> {
    let live = Arc::clone(&state.lock().await.live);
    let map = live.lock().unwrap();
    let sessions = map
        .iter()
        .map(|(id, s)| LiveSessionSummary {
            session_id: id.clone(),
            game_id: s.game_id.clone(),
            strategy: s.strategy.clone(),
            frame_number: s.frame_number,
            max_frames: s.max_frames,
        })
        .collect();
    Json(LiveListResponse { sessions })
}

async fn live_frame(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Response, StatusCode> {
    let live = Arc::clone(&state.lock().await.live);
    let map = live.lock().unwrap();
    let session = map.get(&id).ok_or(StatusCode::NOT_FOUND)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CACHE_CONTROL, "no-store")
        .header("x-frame-number", session.frame_number)
        .header("x-max-frames", session.max_frames)
        .body(Body::from(session.rgba.clone()))
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

#[derive(Serialize)]
struct ReplayResponse {
    session_id: String,
    status: &'static str,
}

/// Decode a recording server-side at real-time pace and publish its frames
/// through the live-session map, so browsers without a working media pipeline
/// can still watch recordings on the canvas viewer.
async fn replay_recording(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<ReplayResponse>, StatusCode> {
    let guard = state.lock().await;
    let recording = guard.db.get_recording(&id).ok_or(StatusCode::NOT_FOUND)?;
    let live = Arc::clone(&guard.live);
    drop(guard);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let session_id = format!("replay_{}_{}", id, timestamp);

    live.lock().unwrap().insert(
        session_id.clone(),
        LiveSession {
            game_id: recording.game_id.clone(),
            strategy: format!("replay:{}", recording.strategy),
            frame_number: 0,
            max_frames: recording.frames,
            rgba: vec![0; 426 * 240 * 4],
        },
    );

    let path = recording.path.clone();
    let sid = session_id.clone();
    std::thread::spawn(move || {
        use std::io::Read;
        use std::process::{Command, Stdio};

        // -re paces decoding at the file's native frame rate.
        let child = Command::new("ffmpeg")
            .args([
                "-re", "-i", &path,
                "-f", "rawvideo", "-pix_fmt", "rgba",
                "-s", "426x240", "-an", "-",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                eprintln!("replay {}: spawn ffmpeg: {}", sid, e);
                live.lock().unwrap().remove(&sid);
                return;
            }
        };

        let mut stdout = child.stdout.take().unwrap();
        let mut buf = vec![0u8; 426 * 240 * 4];
        let mut frame: u64 = 0;
        loop {
            if stdout.read_exact(&mut buf).is_err() {
                break; // EOF or decode error — replay is over
            }
            frame += 1;
            if frame % 3 == 0 {
                let mut map = live.lock().unwrap();
                match map.get_mut(&sid) {
                    Some(entry) => {
                        entry.frame_number = frame;
                        entry.rgba.copy_from_slice(&buf);
                    }
                    None => break, // externally cancelled
                }
            }
        }
        live.lock().unwrap().remove(&sid);
        let _ = child.kill();
        let _ = child.wait();
    });

    Ok(Json(ReplayResponse { session_id, status: "started" }))
}

async fn list_strategies() -> impl IntoResponse {
    Json(StrategiesResponse {
        strategies: vec!["explorer", "passive", "aggressive", "prime"],
    })
}
