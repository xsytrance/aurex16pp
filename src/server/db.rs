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

#[derive(Serialize, Deserialize, Default)]
struct Snapshot {
    games: Vec<Game>,
    recordings: Vec<Recording>,
}

pub struct Database {
    path: String,
    games: Mutex<HashMap<String, Game>>,
    recordings: Mutex<HashMap<String, Recording>>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, String> {
        let mut snapshot: Snapshot = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        // Drop recordings whose MP4 no longer exists on disk.
        snapshot
            .recordings
            .retain(|r| std::path::Path::new(&r.path).exists());

        // A restart can leave games stuck in "generating"; their session is gone.
        for game in &mut snapshot.games {
            if game.status == "generating" {
                game.status = "ready".to_string();
            }
        }

        Ok(Self {
            path: path.to_string(),
            games: Mutex::new(snapshot.games.into_iter().map(|g| (g.id.clone(), g)).collect()),
            recordings: Mutex::new(
                snapshot.recordings.into_iter().map(|r| (r.id.clone(), r)).collect(),
            ),
        })
    }

    fn persist(&self) {
        let snapshot = Snapshot {
            games: self.games.lock().unwrap().values().cloned().collect(),
            recordings: self.recordings.lock().unwrap().values().cloned().collect(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
            let _ = std::fs::write(&self.path, json);
        }
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
        self.persist();
        Ok(game)
    }

    pub fn list_games(&self) -> Vec<Game> {
        self.games.lock().unwrap().values().cloned().collect()
    }

    pub fn get_game(&self, id: &str) -> Option<Game> {
        self.games.lock().unwrap().get(id).cloned()
    }

    pub fn update_game_status(&self, id: &str, status: &str) -> Result<(), String> {
        let found = {
            let mut games = self.games.lock().unwrap();
            match games.get_mut(id) {
                Some(game) => {
                    game.status = status.to_string();
                    true
                }
                None => false,
            }
        };
        if found {
            self.persist();
            Ok(())
        } else {
            Err("Game not found".to_string())
        }
    }

    pub fn add_recording(&self, recording: Recording) -> Result<(), String> {
        self.recordings.lock().unwrap().insert(recording.id.clone(), recording);
        self.persist();
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
