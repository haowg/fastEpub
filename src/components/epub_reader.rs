use dioxus::prelude::*;
use dioxus::prelude::Fragment;
use std::path::PathBuf;
use epub::doc::EpubDoc;
use epub::doc::NavPoint;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::collections::HashMap;

#[derive(Debug)]
struct BookState {
    chapters: Vec<Chapter>,
    metadata: BookMetadata,
    toc: Vec<NavPoint>,  // 新增目录结构
    path_to_chapter: std::collections::HashMap<PathBuf, usize>, // 新增：路径到章节索引的映射
    images: HashMap<String, String>,  // 添加图片资源映射
    raw_chapters: HashMap<String, (String, PathBuf)>,  // 添加原始章节内容存储
}

#[derive(Debug, Clone, PartialEq)]
struct Chapter {
    id: String,
    content: String,
    path: PathBuf,
    play_order: usize,
    processed: bool,  // 添加标记表示是否已处理
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

// 新增 TocItem 组件用于递归渲染目录
#[component]
fn TocItem(
    entry: NavPoint,
    depth: usize,
    current_chapter: Signal<usize>,
    collapsed_nodes: Signal<HashSet<String>>,
    book_state: Signal<BookState>,
    goto_chapter: EventHandler<usize>,
    toggle_collapse: EventHandler<String>,
) -> Element {
    let entry_label = entry.label.clone();
    let node_id = use_memo(move || entry_label.clone());
    let entry_label = entry.label.clone(); // Clone again for later use
    let is_collapsed = collapsed_nodes.read().contains(&node_id());
    let has_children = !entry.children.is_empty();
    let chapter_index = resolve_chapter_index(&entry.content, &book_state.read().path_to_chapter);
    
    let class_name = match chapter_index {
        Some(ci) if ci == *current_chapter.read() => "flex-1 cursor-pointer text-left py-1 text-blue-700",
        _ => "flex-1 cursor-pointer text-left py-1 text-gray-700 hover:bg-gray-400",
    };

    rsx! {
        div {
            class: "flex flex-col",
            // 章节标题行
            div {
                class: "flex items-center gap-1",
                style: "padding-left: {depth * 12}px",
                // 折叠按钮
                {has_children.then(|| rsx!(
                    button {
                        class: "w-4 text-gray-500 hover:text-gray-700 focus:outline-none",
                        onclick: move |_| toggle_collapse.call(node_id.read().clone()),
                        span {
                            class: "inline-block transition-transform duration-200",
                            style: if is_collapsed { "" } else { "transform: rotate(90deg)" },
                            "▶"
                        }
                    }
                ))}
                // 章节标题
                div {
                    class: "{class_name}",
                    onclick: move |_| if let Some(ci) = chapter_index { goto_chapter.call(ci) },
                    "{entry.label}"
                }
            }

            // 递归渲染子节点
            {(!is_collapsed && has_children).then(|| rsx!(
                div {
                    class: "flex flex-col",
                    {entry.children.iter().map(|child| rsx!(
                        TocItem {
                            key: "{child.label}",
                            entry: child.clone(),
                            depth: depth + 1,
                            current_chapter: current_chapter,
                            collapsed_nodes: collapsed_nodes,
                            book_state: book_state,
                            goto_chapter: goto_chapter,
                            toggle_collapse: toggle_collapse,
                        }
                    ))}
                }
            ))}
        }
    }
}

#[component]
pub fn EpubReader(current_file: Signal<String>) -> Element {
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
        images: HashMap::new(),
        raw_chapters: HashMap::new(),  // 添加原始章节内容存储
    });

    let mut current_chapter = use_signal(|| 0);
    let mut load_error = use_signal(|| None::<String>);
    let mut sidebar_width = use_signal(|| 192.0); // 初始宽度 192px (w-48)
    let mut is_resizing = use_signal(|| false);
    let mut preview_width = use_signal(|| 192.0);
    let mut show_preview = use_signal(|| false);
    let mut collapsed_nodes = use_signal(|| HashSet::new()); // 添加折叠状态管理

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

    // 只保留这一个 use_effect，并改进其逻辑
    use_effect(move || {
        let file_path = current_file.read().to_string();
        if (!file_path.is_empty() && file_path.ends_with(".epub")) {
            current_chapter.set(0); // 重置章节索引
            match load_epub(&file_path, book_state.clone()) {
                Ok(_) => load_error.set(None),
                Err(e) => load_error.set(Some(e.to_string())),
            }
        }
        ()
    });

    let chapter = {
        let mut chapter = {
            let state = book_state.read();
            state.chapters.get(*current_chapter.read())
                .cloned()
                .unwrap_or_else(|| Chapter {
                    id: String::new(),
                    content: String::new(),
                    path: PathBuf::new(),
                    play_order: 0,
                    processed: true,
                })
        };
        
        // 如果章节未处理，现在处理它
        if !chapter.processed {
            let state = book_state.read();
            if let Some((content, path)) = state.raw_chapters.get(&chapter.id) {
                chapter.content = process_html_content(content, &state.images, path);
                chapter.processed = true;
                drop(state); // Drop the immutable borrow before taking a mutable borrow
                // 更新处理后的章节
                let mut state = book_state.write();
                if let Some(ch) = state.chapters.get_mut(*current_chapter.read()) {
                    ch.content = chapter.content.clone();
                    ch.processed = true;
                }
            }
        }
        
        chapter
    };

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

    // 添加折叠切换处理函数
    let mut toggle_collapse = move |idx: String| {
        let mut nodes = collapsed_nodes.write();
        if nodes.contains(&idx) {
            nodes.remove(&idx);
        } else {
            nodes.insert(idx);
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
                    class: "flex flex-col gap-1 p-2",
                    {book_state.read().toc.iter().map(|entry| rsx!(
                        TocItem {
                            key: "{entry.label}",
                            entry: entry.clone(),
                            depth: 0,
                            current_chapter: current_chapter,
                            collapsed_nodes: collapsed_nodes,
                            book_state: book_state,
                            goto_chapter: move |idx| goto_chapter(idx),
                            toggle_collapse: move |id| toggle_collapse(id),
                        }
                    ))}
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

// 添加处理HTML内容的函数，改进图片路径处理
fn process_html_content(content: &str, images: &HashMap<String, String>, root_path: &PathBuf) -> String {
    let re = regex::Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).unwrap();
    
    re.replace_all(content, |caps: &regex::Captures| {
        let img_src = &caps[1];
        let img_path = if img_src.starts_with("data:") {
            img_src.to_string()
        } else {
            // 尝试多种路径组合来匹配图片
            let possible_paths = vec![
                img_src.to_string(), // 原始路径
                if img_src.starts_with('/') { 
                    img_src[1..].to_string() // 移除开头的斜杠
                } else { 
                    img_src.to_string() 
                },
                format!("{}", img_src.split('/').last().unwrap_or(img_src)), // 仅文件名
            ];


            // 尝试所有可能的路径并返回找到的图片路径或默认图片
            let default_image = String::from("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA...");
            match possible_paths.iter()
                .find(|path| images.contains_key(*path))
                .and_then(|path| images.get(path)) {
                    Some(found_image) => found_image.clone(),
                    None => {
                        println!("Image not found: {}", img_src); // 调试信息
                        default_image
                    }
                }
        };
        
        format!(r#"<img src="{}" loading="lazy""#, img_path)
    }).to_string()
}

#[allow(dead_code)]
fn load_epub(path: &str, mut book_state: Signal<BookState>) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = EpubDoc::new(path)?;
    let mut chapters = Vec::new();
    let mut raw_chapters = HashMap::new();
    let mut path_to_chapter = std::collections::HashMap::new();
    let mut images = HashMap::new();

    // 改进图片资源加载
    let resource_keys: Vec<_> = doc.resources.keys().cloned().collect();
    for id in resource_keys {
        if let Some((data, mime)) = doc.get_resource(&id) {
            let path = doc.resources.get(&id).unwrap().0.clone();
            if let Some((data, _)) = doc.get_resource(&id) {
                let base64_data = BASE64.encode(&data);
                let data_url = format!("data:{};base64,{}", mime, base64_data);
                
                // 存储多个路径版本以增加匹配成功率
                images.insert(path.to_string_lossy().to_string(), data_url.clone());
                if let Some(file_name) = path.file_name() {
                    images.insert(file_name.to_string_lossy().to_string(), data_url.clone());
                }
                if let Some(path_str) = path.to_str() {
                    if path_str.starts_with('/') {
                        images.insert(path_str[1..].to_string(), data_url.clone());
                    }
                }
            }
        }
    }

    // 修改章节加载逻辑 - 现在只存储原始内容
    for spine_index in 0..doc.spine.len() {
        let id = doc.spine[spine_index].clone();
        if let Some((chapter_content, _mime)) = doc.get_resource(&id) {
            if let Ok(content) = String::from_utf8(chapter_content) {
                let path = doc.resources.get(&id)
                    .map(|(p, _)| p.clone())
                    .unwrap_or_default();
                
                // 存储原始内容
                raw_chapters.insert(id.clone(), (content.clone(), path.clone()));
                
                path_to_chapter.insert(path.clone(), spine_index);
                
                // 章节只存储基本信息
                chapters.push(Chapter {
                    id: id.clone(),
                    content: String::new(), // 暂时为空
                    path,
                    play_order: spine_index,
                    processed: false,
                });
            }
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
        raw_chapters, // 添加原始章节内容
        metadata,
        toc: doc.toc.clone(),
        path_to_chapter,
        images,  // 添加图片资源
    });
    
    Ok(())
}
