use std::sync::Mutex;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub genre: String,
    pub description: String,
    pub status: String,  // "pending" | "generating" | "ready" | "failed"
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub game_id: String,
    pub path: String,
    pub strategy: String,
    pub frames: u64,
    pub duration_secs: f64,
    pub file_size: u64,
    pub created_at: DateTime<Utc>,
}

pub struct Database {
    games: Mutex<HashMap<String, Game>>,
    recordings: Mutex<HashMap<String, Recording>>,
}

impl Database {
    pub fn new(_path: &str) -> Result<Self, String> {
        Ok(Self {
            games: Mutex::new(HashMap::new()),
            recordings: Mutex::new(HashMap::new()),
        })
    }

    pub fn create_game(&self, title: &str, genre: &str, description: &str) -> Result<Game, String> {
        let id = title.to_lowercase().replace(" ", "_").replace(|c: char| !c.is_alphanumeric() && c != '_', "");
        let game = Game {
            id: id.clone(),
            title: title.to_string(),
            genre: genre.to_string(),
            description: description.to_string(),
            status: "pending".to_string(),
            created_at: Utc::now(),
        };
        self.games.lock().unwrap().insert(id, game.clone());
        Ok(game)
    }

    pub fn list_games(&self) -> Vec<Game> {
        self.games.lock().unwrap().values().cloned().collect()
    }

    pub fn get_game(&self, id: &str) -> Option<Game> {
        self.games.lock().unwrap().get(id).cloned()
    }

    pub fn update_game_status(&self, id: &str, status: &str) -> Result<(), String> {
        let mut games = self.games.lock().unwrap();
        if let Some(game) = games.get_mut(id) {
            game.status = status.to_string();
            Ok(())
        } else {
            Err("Game not found".to_string())
        }
    }

    pub fn add_recording(&self, recording: Recording) -> Result<(), String> {
        self.recordings.lock().unwrap().insert(recording.id.clone(), recording);
        Ok(())
    }

    pub fn list_recordings(&self) -> Vec<Recording> {
        self.recordings.lock().unwrap().values().cloned().collect()
    }

    pub fn list_recordings_for_game(&self, game_id: &str) -> Vec<Recording> {
        self.recordings.lock().unwrap()
            .values()
            .filter(|r| r.game_id == game_id)
            .cloned()
            .collect()
    }

    pub fn get_recording(&self, id: &str) -> Option<Recording> {
        self.recordings.lock().unwrap().get(id).cloned()
    }
}
