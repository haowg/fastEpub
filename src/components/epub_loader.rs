use epub::doc::{EpubDoc, NavPoint};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use dioxus::prelude::*;
use crate::components::html_processor::process_html_content;

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
    pub chapter_count: usize,  // 新增字段
}

#[derive(Debug)]
pub struct BookState {
    pub metadata: BookMetadata,
    pub toc: Vec<NavPoint>,
    pub current_content: String,  // 新增字段存储当前章节内容
}

impl BookState {
    pub fn get_chapter(doc: &mut EpubDoc<std::io::BufReader<std::fs::File>>, chapter_idx: usize) -> Chapter {
        let default_chapter = Chapter {
            id: String::new(),
            content: String::from("章节加载失败"),  // 更友好的错误提示
            path: PathBuf::new(),
            play_order: 0,
            processed: true,
        };

        if chapter_idx >= doc.spine.len() {
            return default_chapter;
        }

        let id = doc.spine[chapter_idx].clone();
        if let Some((chapter_content, _mime)) = doc.get_resource(&id) {
            if let Ok(content) = String::from_utf8(chapter_content) {
                // let processed_content = process_html_content(&content);  // 处理HTML内容
                let path = doc.resources.get(&id)
                    .map(|(p, _)| p.clone())
                    .unwrap_or_default();
                
                return Chapter {
                    id,
                    // content: processed_content,
                    content: content,
                    path,
                    play_order: chapter_idx,
                    processed: true,
                };
            }
        }

        default_chapter
    }
}

pub struct EpubLoader {
    doc: EpubDoc<std::io::BufReader<std::fs::File>>,
}

impl EpubLoader {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            doc: EpubDoc::new(path)?,
        })
    }

    pub fn load_book_state(self) -> (BookState, EpubDoc<std::io::BufReader<std::fs::File>>) {
        let chapter_count = self.doc.spine.len();
        let book_state = BookState {
            metadata: BookMetadata {
                title: self.doc.mdata("title"),
                author: self.doc.mdata("creator"),
                description: self.doc.mdata("description"),
                cover_id: self.doc.get_cover_id(),
                chapter_count,  // 添加章节总数
            },
            toc: self.doc.toc.clone(),
            current_content: String::new(),  // 初始化当前章节内容
        };
        (book_state, self.doc)
    }
}

pub fn load_epub(path: &str, mut book_state: Signal<BookState>) -> Result<(), Box<dyn std::error::Error>> {
    let loader = EpubLoader::new(path)?;
    let (new_state, mut doc) = loader.load_book_state();
    
    // 加载第一章内容
    let content = BookState::get_chapter(&mut doc, 0).content;
    
    // 设置新状态
    book_state.set(BookState {
        metadata: new_state.metadata,
        toc: new_state.toc,
        current_content: content,
    });
    
    Ok(())
}