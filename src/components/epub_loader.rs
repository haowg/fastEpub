use epub::doc::{EpubDoc, NavPoint};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use dioxus::prelude::*;
// use crate::components::html_processor::process_html_content;
use std::time::Instant;
use std::fs::File;
use std::io::{Read, Seek};

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
    pub unique_identifier: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub cover_id: Option<String>,
    pub chapter_count: usize,
}

impl From<&BookContent> for BookMetadata {
    fn from(content: &BookContent) -> Self {
        Self {
            unique_identifier: content.unique_identifier.clone(),
            title: content.metadata.get("title").and_then(|v| v.first()).cloned(),
            author: content.metadata.get("creator").and_then(|v| v.first()).cloned(),
            description: content.metadata.get("description").and_then(|v| v.first()).cloned(),
            cover_id: content.cover_id.clone(),
            chapter_count: content.spine.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BookState {
    pub metadata: BookMetadata,
    pub toc: Vec<NavPoint>,
    pub content: BookContent,  // Add content field
}

impl BookState {
    pub fn empty() -> Self {
        Self {
            metadata: BookMetadata {
                unique_identifier: None,
                title: None,
                author: None,
                description: None,
                cover_id: None,
                chapter_count: 0,
            },
            toc: Vec::new(),
            content: BookContent::empty(),
        }
    }

    pub fn get_chapter(&self, chapter_idx: usize) -> Chapter {
        // Move get_chapter logic here
        let default_chapter = Chapter {
            id: String::new(),
            content: String::from("章节加载失败"),
            path: PathBuf::new(),
            play_order: 0,
            processed: true,
        };
        println!("Getting chapter: {}", chapter_idx);

        if chapter_idx >= self.content.spine.len() {
            return default_chapter;
        }
        println!("pass length check {}", chapter_idx);

        let id = &self.content.spine[chapter_idx];
        if let Some(content) = self.content.resource_content.get(id) {
            if let Ok(content_str) = String::from_utf8(content.clone()) {
                let path = self.content.resources.get(id)
                    .map(|(p, _)| p.clone())
                    .unwrap_or_default();
                
                return Chapter {
                    id: id.clone(),
                    content: content_str,
                    path,
                    play_order: chapter_idx,
                    processed: true,
                };
            }
        }
        default_chapter
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BookContent {
    pub current: usize,
    pub spine: Vec<String>,
    pub resources: HashMap<String, (PathBuf, String)>,
    pub resource_content: HashMap<String, Vec<u8>>,
    pub toc: Vec<NavPoint>,
    pub metadata: HashMap<String, Vec<String>>,
    pub root_base: PathBuf,
    pub root_file: PathBuf,
    pub extra_css: Vec<String>,
    pub unique_identifier: Option<String>,
    pub cover_id: Option<String>,
}

impl BookContent {
    pub fn empty() -> Self {
        Self {
            current: 0,
            spine: Vec::new(),
            resources: HashMap::new(),
            resource_content: HashMap::new(),
            toc: Vec::new(),
            metadata: HashMap::new(),
            root_base: PathBuf::new(),
            root_file: PathBuf::new(),
            extra_css: Vec::new(),
            unique_identifier: None,
            cover_id: None,
        }
    }

    fn read_all_resources<R: Read + Seek>(doc: &mut EpubDoc<R>) -> HashMap<String, Vec<u8>> {
        let mut resource_content = HashMap::new();
        for id in doc.spine.clone() {
            if let Some((content, _mime)) = doc.get_resource(&id) {
                resource_content.insert(id, content);
            }
        }
        resource_content
    }

    fn from_epub<R: Read + Seek>(mut doc: EpubDoc<R>) -> Result<Self, Box<dyn std::error::Error>> {
        // Read all data first
        let resource_content = Self::read_all_resources(&mut doc);
        
        // Debug print TOC data
        println!("Loading TOC with {} entries", doc.toc.len());
        
        // Clone necessary data
        let content = Self {
            current: 0,
            spine: doc.spine.clone(),
            resources: doc.resources.clone(),
            resource_content,
            toc: doc.toc.clone(),
            metadata: doc.metadata.clone(),
            root_base: doc.root_base.clone(),
            root_file: doc.root_file.clone(),
            extra_css: doc.extra_css.clone(),
            unique_identifier: doc.unique_identifier.clone(),
            cover_id: doc.cover_id.clone(),
        };

        // Verify TOC data
        println!("Created BookContent with {} TOC entries", content.toc.len());
        
        // Explicitly drop doc here
        drop(doc);
        
        Ok(content)
    }
}

pub fn load_epub(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading epub: {}", path);
    let mut book_state = use_context::<Signal<BookState>>();
    let start = Instant::now();
    
    let book_content = {
        let doc = EpubDoc::new(path)?;
        println!("Initial TOC size: {}", doc.toc.len());
        BookContent::from_epub(doc)?
    };

    book_state.set(BookState {
        metadata: (&book_content).into(),
        toc: book_content.toc.clone(),
        content: book_content,
    });
    
    println!("Load completed in: {:?}", start.elapsed());
    Ok(())
}
