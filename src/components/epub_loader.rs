use epub::doc::{EpubDoc, NavPoint};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Chapter {
    pub id: String,
    pub content: String,
    pub path: PathBuf,
    pub play_order: usize,
    pub processed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BookMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub cover_id: Option<String>,
}

#[derive(Debug)]
pub struct BookState {
    pub chapters: Vec<Chapter>,
    pub metadata: BookMetadata,
    pub toc: Vec<NavPoint>,
    pub path_to_chapter: HashMap<PathBuf, usize>,
    pub images: HashMap<String, String>,
    pub raw_chapters: HashMap<String, (String, PathBuf)>,
}

pub struct EpubLoader {
    doc: EpubDoc<std::io::BufReader<std::fs::File>>,
    images: HashMap<String, String>,
}

impl EpubLoader {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            doc: EpubDoc::new(path)?,
            images: HashMap::new(),
        })
    }

    pub fn load_images(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let resource_keys: Vec<_> = self.doc.resources.keys().cloned().collect();
        for id in resource_keys {
            if let Some((data, mime)) = self.doc.get_resource(&id) {
                let path = self.doc.resources.get(&id).unwrap().0.clone();
                let base64_data = BASE64.encode(&data);
                let data_url = format!("data:{};base64,{}", mime, base64_data);
                
                // Store multiple path versions
                self.images.insert(path.to_string_lossy().to_string(), data_url.clone());
                if let Some(file_name) = path.file_name() {
                    self.images.insert(file_name.to_string_lossy().to_string(), data_url.clone());
                }
                if let Some(path_str) = path.to_str() {
                    if path_str.starts_with('/') {
                        self.images.insert(path_str[1..].to_string(), data_url);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_book_state(&mut self) -> Result<BookState, Box<dyn std::error::Error>> {
        let mut chapters = Vec::new();
        let mut raw_chapters = HashMap::new();
        let mut path_to_chapter = HashMap::new();

        // Load chapters structure first
        for spine_index in 0..self.doc.spine.len() {
            let id = self.doc.spine[spine_index].clone();
            if let Some((chapter_content, _mime)) = self.doc.get_resource(&id) {
                if let Ok(content) = String::from_utf8(chapter_content) {
                    let path = self.doc.resources.get(&id)
                        .map(|(p, _)| p.clone())
                        .unwrap_or_default();
                    
                    raw_chapters.insert(id.clone(), (content, path.clone()));
                    path_to_chapter.insert(path.clone(), spine_index);
                    
                    chapters.push(Chapter {
                        id: id.clone(),
                        content: String::new(),
                        path,
                        play_order: spine_index,
                        processed: false,
                    });
                }
            }
        }

        let metadata = BookMetadata {
            title: self.doc.mdata("title"),
            author: self.doc.mdata("creator"),
            description: self.doc.mdata("description"),
            cover_id: self.doc.get_cover_id(),
        };

        Ok(BookState {
            chapters,
            raw_chapters,
            metadata,
            toc: self.doc.toc.clone(),
            path_to_chapter,
            images: self.images.clone(),
        })
    }
}

pub fn load_epub(path: &str, mut book_state: Signal<BookState>) -> Result<(), Box<dyn std::error::Error>> {
    let mut loader = EpubLoader::new(path)?;
    loader.load_images()?;
    let new_state = loader.load_book_state()?;
    book_state.set(new_state);
    Ok(())
}
