use dioxus::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::components::{TableOfContents, BookMetadata, BookState, load_epub, AppState, process_html_content};

#[derive(Props, PartialEq, Clone)]
pub struct ContentViewProps {
    pub content: String,
}

#[component]
pub fn EpubReader() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let current_file = use_context::<Signal<String>>();
    let mut book_state = use_context_provider(|| Signal::new(BookState::empty()));
    let mut current_chapter = use_context_provider(|| Signal::new(0));
    let mut spine_index = use_signal(|| 0); // 改为use_signal
    let mut chapter_content = use_signal(|| String::new());

    // 将 goto_chapter 定义为闭包
    let mut goto_chapter = move |new_chapter: usize| {
        let mut state = app_state.write();
        current_chapter.set(new_chapter);
        
        // 更新spine_index
        if let Some(idx) = book_state.read().content.get_spine_index(new_chapter) {
            spine_index.set(idx);  // 使用set方法更新值
        }
        
        state.update_progress(current_file.read().to_string(), new_chapter);
        let content = book_state.write().get_chapter(new_chapter).content;
        chapter_content.set(content);
    };

    // 将 set_chapter_by_spine 定义为闭包
    let mut set_chapter_by_spine = move |idx: usize| {
        spine_index.set(idx);  // 使用set方法更新值
        let image_cache = book_state.read().image_cache.clone();
        let mut st = book_state.write();
        if let Some(ref mut doc) = st.doc {
            let spine_id = doc.spine.get(idx).cloned();
            if let Some(spine_id) = spine_id {
                if let Some((raw_content, _)) = doc.get_resource(&spine_id) {
                    let processed = process_html_content(
                        &String::from_utf8_lossy(&raw_content),
                        &doc.resources,
                        &image_cache
                    );
                    chapter_content.set(processed);
                }
                if let Some(&play_order) = st.content.spine_to_order.get(&idx) {
                    current_chapter.set(play_order);
                }
            }
        }
    };

    let mut loaded_file = use_signal(|| String::new());
    let mut load_error = use_signal(|| None::<String>);
    let mut sidebar_width = use_signal(|| 192.0);
    let mut is_resizing = use_signal(|| false);
    let mut preview_width = use_signal(|| 192.0);
    let mut show_preview = use_signal(|| false);

    // 缓存当前文件路径
    let current = use_memo(move || current_file.read().to_string());

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

    // 修改加载逻辑
    use_effect(move || {
        let file_path = current.read().to_string();
        if (!file_path.is_empty() && 
           *loaded_file.read() != file_path) {
            
            let saved_chapter = app_state.read().get_progress(&file_path);
            
            match load_epub(&file_path) {
                Ok(_) => {
                    load_error.set(None);
                    loaded_file.set(file_path.clone());
                    
                    let chapter = saved_chapter.unwrap_or(0);
                    
                    // 先设置spine_index为0，防止未初始化状态
                    spine_index.set(0);  // 使用set方法设置初始值

                    current_chapter.set(chapter);
                    
                    // 获取内容并更新spine_index
                    let content = book_state.write().get_chapter(chapter).content;
                    chapter_content.set(content);
                    
                    // 更新spine_index (如果找到对应的索引)
                    if let Some(idx) = book_state.read().content.get_spine_index(chapter) {
                        spine_index.set(idx);  // 使用set方法更新值
                    }
                    
                    // 更新书库
                    let (title, author) = {
                        let state = book_state.read();
                        (
                            state.metadata.title.clone().unwrap_or_else(|| "未知标题".to_string()),
                            state.metadata.author.clone().unwrap_or_else(|| "未知作者".to_string())
                        )
                    };
                    
                    let mut state = app_state.write();
                    if (!state.library.iter().any(|b| b.path == file_path)) {
                        state.add_to_library(
                            file_path.clone(),
                            title,
                            author,
                            chapter
                        );
                    }
                }
                Err(e) => load_error.set(Some(e.to_string())),
            }
        }
    });

    // 修改预计算章节数的方式
    let total_chapters = use_memo(move || {
        book_state.read().metadata.chapter_count
    });

    // 修改go_next和go_prev以添加更多安全检查
    let go_next = move |_| {
        let current = *spine_index.read();
        let max_spine = book_state.read().content.spine.len();
        
        if current < max_spine.saturating_sub(1) {
            set_chapter_by_spine(current + 1);
        }
    };

    let go_prev = move |_| {
        let current = *spine_index.read();
        let new_spine = current.saturating_sub(1);
        if new_spine < current {
            set_chapter_by_spine(new_spine);
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
                // 使用新的目录组件
                TableOfContents {
                    on_select: move |chapter| goto_chapter(chapter)
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
                    content_view {
                        content: chapter_content.read().clone(),
                    }
                }
                // 导航按钮
                div { class: "flex justify-center space-x-4",
                    button {
                        class: "px-4 py-2 bg-gray-300 rounded disabled:opacity-50",
                        disabled: *spine_index.read() == 0,
                        onclick: go_prev,
                        "上一章"
                    }
                    button {
                        class: "px-4 py-2 bg-gray-300 rounded disabled:opacity-50",
                        disabled: *spine_index.read() >= book_state.read().content.spine.len().saturating_sub(1),
                        onclick: go_next,
                        "下一章"
                    }
                }
            }
        }
    }
}
#[component]
pub fn content_view(props: ContentViewProps) -> Element {
    rsx! {
        div {
            class: "flex-1 p-8 overflow-y-auto bg-white text-gray-800 h-full relative",
            dangerous_inner_html: "{props.content}",
            style: "img {{ max-width: 100%; height: auto; display: block; margin: 1em auto; }}"
        }
    }
}
