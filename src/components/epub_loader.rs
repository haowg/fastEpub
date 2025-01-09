use epub::doc::{EpubDoc, NavPoint};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use dioxus::prelude::*;
// use crate::components::html_processor::process_html_content;
use std::time::Instant;
use std::fs::File;
use std::io::{Read, Seek, BufReader};

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
    pub order_path: HashMap<usize, PathBuf>,
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
            order_path: content.order_path.clone(),
        }
    }
}

#[derive(Debug)]
pub struct BookState {
    pub metadata: BookMetadata,
    pub toc: Vec<NavPoint>,
    pub content: BookContent,  // Add content field
    pub doc: Option<EpubDoc<BufReader<File>>>,
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
                order_path: HashMap::new(),
            },
            toc: Vec::new(),
            content: BookContent::empty(),
            doc: None,
        }
    }

    pub fn get_chapter(&mut self, play_order: usize) -> Chapter {
        // If no cached entry, fall back to doc and toc
        if let Some(path) = self.content.order_path.get(&play_order) {
            if let Some(ref mut doc) = self.doc {
                let cleaned_path = path.to_str().unwrap_or("").split("#").next().unwrap_or("");
                if let Some(content) = doc.get_resource_str_by_path(cleaned_path) {
                    return Chapter {
                        id: path.display().to_string(),
                        content,
                        path: path.clone(),
                        play_order: play_order,
                        processed: true,
                    };
                }
            }
            Chapter {
                id: path.display().to_string(),
                content: "无法读取章节".into(),
                path: path.clone(),
                play_order: play_order,
                processed: true,
            }
        } else {
            Chapter {
                id: play_order.to_string(),
                content: "章节不存在".into(),
                path: PathBuf::new(),
                play_order: play_order,
                processed: true,
            }
        }
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
    order_path: HashMap<usize, PathBuf>,
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
            order_path: HashMap::new(),
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
    fn expand_toc(toc: Vec<NavPoint>) -> Vec<(usize, PathBuf)> {
        let mut result = Vec::new();

        // 遍历每个章节
        for nav in toc {
            // 添加当前章节路径
            result.push((nav.play_order, nav.content.clone()));
            
            // 递归展开子章节
            if nav.children.len() > 0 {
                result.extend(Self::expand_toc(nav.children.clone()));
            }
        }

        result
    }

    fn from_epub<R: Read + Seek>(mut doc: EpubDoc<R>) -> Result<Self, Box<dyn std::error::Error>> {
        let resource_content = Self::read_all_resources(&mut doc);
        let chapter_paths: Vec<(usize, PathBuf)> = Self::expand_toc(doc.toc.clone());
        // Then read content for each path
        let mut order_path = HashMap::new();
        for (play_order, path) in chapter_paths {
            order_path.insert(play_order, path);
        }

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
            order_path: order_path,
        };
        
        Ok(content)
    }
}

pub fn load_epub(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut book_state = use_context::<Signal<BookState>>();
    let start = Instant::now();
    
    let book_content = {
        let doc = EpubDoc::new(path)?;
        BookContent::from_epub(doc)?
    };

    book_state.set(BookState {
        metadata: (&book_content).into(),
        toc: book_content.toc.clone(),
        content: book_content,
        doc: Some(EpubDoc::new(path)?),
    });
    
    Ok(())
}
