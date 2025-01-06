use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]  // 添加 PartialEq
pub struct BookInfo {
    pub path: String,
    pub title: String,
    pub author: String,
    pub last_read: chrono::DateTime<chrono::Utc>,
    pub chapter_index: usize,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppState {
    pub last_book: Option<String>,
    pub reading_progress: HashMap<String, ReadingProgress>,
    pub library: Vec<BookInfo>,  // 添加书库
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
        // 更新阅读进度
        self.reading_progress.insert(book_path.clone(), ReadingProgress {
            chapter_index: chapter,
            last_read: chrono::Utc::now(),
        });
        
        // 更新最后打开的书
        self.last_book = Some(book_path.clone());
        
        // 更新书库中的进度
        if let Some(book) = self.library.iter_mut().find(|b| b.path == book_path) {
            book.chapter_index = chapter;
            book.last_read = chrono::Utc::now();
        }

        // 立即保存更改
        if let Err(e) = self.save() {
            eprintln!("Failed to save state: {}", e);
        }
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

    pub fn add_to_library(&mut self, path: String, title: String, author: String, chapter: usize) {
        if !self.library.iter().any(|book| book.path == path) {
            self.library.push(BookInfo {
                path,
                title,
                author,
                last_read: chrono::Utc::now(),
                chapter_index: chapter,
            });
        }
        let _ = self.save();
    }

    pub fn get_library(&self) -> Vec<BookInfo> {
        let mut books = self.library.clone();
        books.sort_by(|a, b| b.last_read.cmp(&a.last_read));
        books
    }
}
