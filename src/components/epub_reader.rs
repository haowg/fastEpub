use dioxus::prelude::*;
use dioxus::prelude::Fragment;
use std::path::PathBuf;
use epub::doc::EpubDoc;
use epub::doc::NavPoint;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct BookState {
    chapters: Vec<Chapter>,
    metadata: BookMetadata,
    toc: Vec<NavPoint>,  // 新增目录结构
    path_to_chapter: std::collections::HashMap<PathBuf, usize>, // 新增：路径到章节索引的映射
}

#[derive(Debug, Clone, PartialEq)]
struct Chapter {
    id: String,
    content: String,
    path: PathBuf,
    play_order: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct BookMetadata {
    title: Option<String>,
    author: Option<String>,
    description: Option<String>,
    cover_id: Option<String>,
}

// 添加辅助函数以解析章节索引
fn resolve_chapter_index(
    original: &PathBuf,
    map: &std::collections::HashMap<PathBuf, usize>,
) -> Option<usize> {
    // 删除潜在锚点 (#...)
    let cleaned = original
        .to_str()
        .and_then(|s| s.split('#').next())
        .map(PathBuf::from)
        .unwrap_or(original.clone());

    // 尝试完整路径
    if let Some(&idx) = map.get(&cleaned) {
        return Some(idx);
    }

    // 尝试仅文件名
    if let Some(file_name) = cleaned.file_name() {
        let file_path = PathBuf::from(file_name);
        if let Some(&idx) = map.get(&file_path) {
            return Some(idx);
        }
    }

    None
}

#[component]
pub fn EpubReader() -> Element {
    let mut book_state = use_signal(|| BookState {
        chapters: Vec::new(),
        metadata: BookMetadata {
            title: None,
            author: None,
            description: None,
            cover_id: None,
        },
        toc: Vec::new(),
        path_to_chapter: std::collections::HashMap::new(),
    });

    let mut current_chapter = use_signal(|| 0);
    let mut load_error = use_signal(|| None::<String>);
    let mut sidebar_width = use_signal(|| 192.0); // 初始宽度 192px (w-48)
    let mut is_resizing = use_signal(|| false);
    let mut preview_width = use_signal(|| 192.0);
    let mut show_preview = use_signal(|| false);
    let mut current_file = use_signal(|| String::from("/home/hwg/Calibre 书库/Ban Du/Can Ming (81)/Can Ming - Ban Du.epub"));

    let on_mouse_down = move |e: Event<MouseData>| {
        is_resizing.set(true);
        show_preview.set(true);
        preview_width.set(e.client_coordinates().x as f64);
    };

    let on_mouse_move = move |e: Event<MouseData>| {
        if *is_resizing.read() {
            let new_width = e.client_coordinates().x as f64;
            if new_width >= 100.0 && new_width <= 400.0 {
                preview_width.set(new_width);
            }
        }
    };

    let on_mouse_up = move |_| {
        if *is_resizing.read() {
            sidebar_width.set(*preview_width.read());
            is_resizing.set(false);
            show_preview.set(false);
        }
    };

    let mut load_new_file = move |path: String| {
        match load_epub(&path, book_state.clone()) {
            Ok(_) => (),
            Err(e) => load_error.set(Some(e.to_string())),
        }
    };

    // 修改初始化加载电子书的逻辑
    use_effect(move || {
        if book_state.read().chapters.is_empty() {
            match load_epub(current_file.read().as_str(), book_state.clone()) {
                Ok(_) => (),
                Err(e) => load_error.set(Some(e.to_string())),
            }
        }
        ()
    });

    let chapter = book_state.read().chapters.get(*current_chapter.read())
        .cloned()
        .unwrap_or_else(|| Chapter {
            id: String::new(),
            content: String::new(),
            path: PathBuf::new(),
            play_order: 0,
        });

    let go_next = move |_| {
        let mut state = current_chapter.write();
        if *state + 1 < book_state.read().chapters.len() {
            *state += 1;
        }
    };

    let go_prev = move |_| {
        let mut state = current_chapter.write();
        if *state > 0 {
            *state -= 1;
        }
    };

    let mut goto_chapter = move |idx: usize| {
        let mut state = current_chapter.write();
        if idx < book_state.read().chapters.len() {
            *state = idx;
        }
    };



    rsx! {
        div {
            onmousemove: on_mouse_move,
            onmouseup: on_mouse_up,
            class: "flex flex-1 overflow-hidden h-[calc(100vh-48px)] relative",
            
            // 拖动时的全屏遮罩层
            if *is_resizing.read() {
                div {
                    class: "fixed inset-0 bg-transparent cursor-col-resize select-none",
                    style: "z-index: 9999",
                }
            }

            // 侧边栏
            div {
                class: "bg-gray-200 overflow-y-auto h-full relative",
                style: "width: {sidebar_width}px; z-index: 1",
                // 书籍信息
                div { class: "text-center",
                    h1 { class: "text-xl font-bold",
                        "{book_state.read().metadata.title.as_deref().unwrap_or(\"未知标题\")}"
                    }
                    p { class: "text-sm",
                        "作者: {book_state.read().metadata.author.as_deref().unwrap_or(\"未知作者\")}"
                    }
                },
                // 目录
                div {
                    {book_state.read().toc.clone().into_iter().enumerate().map(|(idx, entry)| {
                        // 从路径映射获取实际章节索引
                        let chapter_index = resolve_chapter_index(&entry.content, &book_state.read().path_to_chapter);
                        
                        let class_name = match chapter_index {
                            Some(ci) if ci == *current_chapter.read() => "flex-1 block cursor-pointer text-left py-1 text-blue-700",
                            _ => "flex-1 block cursor-pointer text-left py-1 text-gray-700 hover:bg-gray-400",
                        };
                        
                        rsx! {
                            div {
                                class: "{class_name}",
                                key: "{idx}",
                                onclick: move |_| {
                                    if let Some(ci) = chapter_index {
                                        goto_chapter(ci);
                                    }
                                },
                                "{entry.label}"
                            }

                            if !entry.children.is_empty() {
                                div {
                                    class: "pl-4",
                                    {entry.children.clone().into_iter().enumerate().map(|(child_idx, child)| {
                                        let child_chapter_index = resolve_chapter_index(&child.content, &book_state.read().path_to_chapter);
                                        
                                        let child_class_name = match child_chapter_index {
                                            Some(ci) if ci == *current_chapter.read() => "flex-1 block cursor-pointer text-left py-1 text-blue-700",
                                            _ => "flex-1 block cursor-pointer text-left py-1 text-gray-700 hover:bg-gray-400",
                                        };
                                        
                                        rsx! {
                                            div {
                                                class: "{child_class_name}",
                                                key: "{idx}-{child_idx}",
                                                onclick: move |_| {
                                                    if let Some(ci) = child_chapter_index {
                                                        goto_chapter(ci);
                                                    }
                                                },
                                                "{child.label}"
                                            }

                                        }
                                    }).collect::<Vec<_>>().into_iter()}
                                }
                            }
                        }
                    }).collect::<Vec<_>>().into_iter()}
                }
            }

            // 拖动条
            div {
                class: "w-1 cursor-col-resize bg-gray-300 hover:bg-gray-400 active:bg-gray-500 relative",
                style: "z-index: 2",
                onmousedown: on_mouse_down,
            }

            // 预览层
            if *show_preview.read() {
                div {
                    class: "absolute top-0 bottom-0 border-r-4 border-gray-400 pointer-events-none",
                    style: "left: {preview_width}px; z-index: 9999",
                }
            },

            // 内容区域
            div { 
                class: "flex-1 p-8 overflow-y-auto bg-white text-gray-800 h-full relative",
                style: "z-index: 1",
                if let Some(error) = load_error.read().as_ref() {
                    div { class: "text-red-500", "{error}" }
                } else {
                    div {
                        dangerous_inner_html: "{chapter.content}"
                    }
                }
                // 导航按钮
                div { class: "flex justify-center space-x-4",
                    button {
                        class: "px-4 py-2 bg-gray-300 rounded disabled:opacity-50",
                        disabled: *current_chapter.read() == 0,
                        onclick: go_prev,
                        "上一章"
                    }
                    button {
                        class: "px-4 py-2 bg-gray-300 rounded disabled:opacity-50",
                        disabled: *current_chapter.read() >= book_state.read().chapters.len().saturating_sub(1),
                        onclick: go_next,
                        "下一章"
                    }
                },
                // 加载新文件按钮
                button {
                    class: "px-4 py-2 bg-blue-500 text-white rounded",
                    onclick: move |_| load_new_file(String::from("/home/hwg/文档/小说/《少年阿宾》（全本+外篇） (Ben [Ben]) (Z-Library).epub")),
                    "加载新书"
                }
            }
        }
    }
}

#[allow(dead_code)]
fn load_epub(path: &str, mut book_state: Signal<BookState>) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = EpubDoc::new(path)?;
    let mut chapters = Vec::new();
    let mut path_to_chapter = std::collections::HashMap::new();
    let mut path_to_spine = std::collections::HashMap::new();

    // 首先建立 spine path 到位置的映射
    for (spine_index, id) in doc.spine.iter().enumerate() {
        if let Some(path) = doc.resources.get(id).map(|(p, _)| p.clone()) {
            path_to_spine.insert(path, spine_index);
        }
    }

    // 加载章节内容
    for spine_index in 0..doc.spine.len() {
        let id = doc.spine[spine_index].clone();
        if let Some((chapter_content, _mime)) = doc.get_resource(&id) {
            let content = String::from_utf8(chapter_content)?;
            let path = doc.resources.get(&id)
                .map(|(p, _)| p.clone())
                .unwrap_or_default();
            
            // 保存路径到章节索引的映射
            path_to_chapter.insert(path.clone(), spine_index);
            
            chapters.push(Chapter {
                id: id.clone(),
                content,
                path,
                play_order: spine_index,
            });
        }
    }

    let metadata = BookMetadata {
        title: doc.mdata("title"),
        author: doc.mdata("creator"),
        description: doc.mdata("description"),
        cover_id: doc.get_cover_id(),
    };

    book_state.set(BookState {
        chapters,
        metadata,
        toc: doc.toc.clone(),
        path_to_chapter,
    });
    
    Ok(())
}
