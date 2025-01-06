use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
pub struct AppState {
    pub last_book: Option<String>,
    pub last_chapter: Option<usize>,
    pub reading_progress: HashMap<String, ReadingProgress>,  // 添加阅读进度记录
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ReadingProgress {
    pub chapter_index: usize,
    pub last_read: chrono::DateTime<chrono::Utc>,
}

const STATE_FILE: &str = "app_state.json";

impl AppState {
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("fast_epub"))
            .unwrap_or_else(|| PathBuf::from("."));

        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }

        let state_path = config_dir.join(STATE_FILE);
        fs::read_to_string(state_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("fast_epub"))
            .unwrap_or_else(|| PathBuf::from("."));

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let state_path = config_dir.join(STATE_FILE);
        let content = serde_json::to_string(self)?;
        fs::write(state_path, content)?;
        Ok(())
    }

    pub fn update_progress(&mut self, book_path: String, chapter: usize) {
        self.reading_progress.insert(book_path, ReadingProgress {
            chapter_index: chapter,
            last_read: chrono::Utc::now(),
        });
        let _ = self.save();
    }

    pub fn get_progress(&self, book_path: &str) -> Option<usize> {
        self.reading_progress.get(book_path).map(|p| p.chapter_index)
    }

    pub fn get_last_book(&self) -> Option<(String, usize)> {
        self.last_book.as_ref().and_then(|path| {
            self.reading_progress
                .get(path)
                .map(|progress| (path.clone(), progress.chapter_index))
        })
    }
}
