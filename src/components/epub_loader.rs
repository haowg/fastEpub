use epub::doc::{EpubDoc, NavPoint};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use dioxus::prelude::*;
use crate::components::html_processor::process_html_content;
use std::time::Instant;
use std::fs::File;
use std::io::{Read, Seek, BufReader};
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use std::path::Path;

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
    pub spine_to_order: HashMap<usize, usize>,  // Add this field
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
            spine_to_order: content.spine_to_order.clone(),
        }
    }

}

#[derive(Debug)]
pub struct BookState {
    pub metadata: BookMetadata,
    pub toc: Vec<NavPoint>,
    pub content: BookContent,  // Add content field
    pub doc: Option<EpubDoc<BufReader<File>>>,
    pub image_cache: HashMap<String, String>,  // 改为存储 base64 字符串
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
                spine_to_order: HashMap::new(),
            },
            toc: Vec::new(),
            content: BookContent::empty(),
            doc: None,
            image_cache: HashMap::new(),
        }
    }

    fn get_mime_type(path: &Path) -> Option<&'static str> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "svg" => "image/svg+xml",
                _ => "image/jpeg",  // 默认为jpeg
            })
    }

    fn is_image_path(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(), 
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | 
                "bmp" | "tiff" | "tif" | "ico"
            )
        } else {
            false
        }
    }

    fn cache_images(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut doc) = self.doc {
            let mut image_cache = HashMap::new();

            let resources = doc.resources.clone();
            for (id, (path, mime)) in resources.iter() {
                // 同时检查 mime 类型和文件路径
                if mime.starts_with("image/") || Self::is_image_path(path) {
                    if let Some((data, _)) = doc.get_resource(id) {
                        let actual_mime = if mime.starts_with("image/") {
                            mime.clone()
                        } else {
                            Self::get_mime_type(path).unwrap_or("image/jpeg").to_string()
                        };

                        let base64_str = format!(
                            "data:{};base64,{}", 
                            actual_mime,
                            general_purpose::STANDARD.encode(&data)
                        );

                        // 存储多种路径格式
                        if let Some(path_str) = path.to_str() {
                            // 完整路径
                            let clean_path = path_str.replace("\\", "/");
                            image_cache.insert(clean_path.clone(), base64_str.clone());
                            
                            // 相对路径 (去掉 OEBPS)
                            if let Some(rel_path) = clean_path.strip_prefix("OEBPS/") {
                                image_cache.insert(rel_path.to_string(), base64_str.clone());
                            }
                            
                            // 文件名
                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                image_cache.insert(file_name.to_string(), base64_str.clone());
                            }
                        }
                    }
                }
            }

            self.image_cache = image_cache;
        }
        Ok(())
    }

    pub fn get_chapter(&mut self, play_order: usize) -> Chapter {
        // If no cached entry, fall back to doc and toc
        if let Some(path) = self.content.order_path.get(&play_order) {
            if let Some(ref mut doc) = self.doc {
                let cleaned_path = path.to_str().unwrap_or("").split("#").next().unwrap_or("");
                if let Some(content) = doc.get_resource_str_by_path(cleaned_path) {
                    // 修改：将资源基础路径传递给处理函数
                    let processed_content = process_html_content(
                        &content, 
                        &doc.resources,
                        &self.image_cache  // 传递图片缓存而不是root_base
                    );
                    return Chapter {
                        id: path.display().to_string(),
                        content: processed_content,
                        path: path.clone(),
                        play_order,
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
    pub order_path: HashMap<usize, PathBuf>,
    pub spine_to_order: HashMap<usize, usize>,  // Add this field
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
            spine_to_order: HashMap::new(),
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
        
        let mut order_path = HashMap::new();
        let mut spine_to_order = HashMap::new();
        
        // Build the mapping between spine index and play_order
        for (play_order, path) in chapter_paths.iter() {
            order_path.insert(*play_order, path.clone());
            
            // Get clean path without anchor
            let clean_path = path.to_str()
                .unwrap_or("")
                .split("#")
                .next()
                .unwrap_or("");
                
            // Find matching resource ID from resources map
            if let Some((resource_id, _)) = doc.resources.iter()
                .find(|(_, (res_path, _))| {
                    res_path.to_str()
                        .unwrap_or("")
                        .contains(clean_path)
                }) {
                // Find this resource ID in spine
                if let Some(spine_idx) = doc.spine.iter().position(|id| id == resource_id) {
                    spine_to_order.insert(spine_idx, *play_order);
                }
            }
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
            spine_to_order,
        };
        
        Ok(content)
    }

    pub fn get_spine_index(&self, play_order: usize) -> Option<usize> {
        self.spine_to_order
            .iter()
            .find(|(_spine_idx, &order)| order == play_order)
            .map(|(&spine_idx, _)| spine_idx)
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
        image_cache: HashMap::new(),
    });

    let mut state = book_state.write();
    state.cache_images()?;  // 缓存图片为base64
    
    Ok(())
}
