use dioxus::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::components::{TableOfContents, BookMetadata, BookState, load_epub, AppState, process_html_content};

pub fn goto_chapter(
    new_chapter: usize,
) {
    let book_state = use_context::<Signal<BookState>>();
    let mut app_state = use_context::<Signal<AppState>>();
    let current_file = use_context::<Signal<String>>();

    let total = book_state.read().metadata.chapter_count;
    if new_chapter < total {
        let mut state = app_state.write();
        let mut current_chapter = use_context::<Signal<usize>>();
        current_chapter.set(new_chapter);
        state.update_progress(current_file.read().to_string(), new_chapter);
    }
}

#[component]
pub fn EpubReader() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let current_file = use_context::<Signal<String>>();
    let book_state = use_context_provider(|| Signal::new(BookState::empty()));
    let mut current_chapter = use_context_provider(|| Signal::new(0));

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
           *loaded_file.read() != file_path) { // 只在文件变化时加载
            
            let saved_chapter = app_state.read().get_progress(&file_path);
            
            match load_epub(&file_path) {
                Ok(_) => {
                    load_error.set(None);
                    loaded_file.set(file_path.clone()); // 更新已加载文件
                    
                    // 更新书库和章节位置
                    let (title, author) = {
                        let state = book_state.read();
                        (
                            state.metadata.title.clone().unwrap_or_else(|| "未知标题".to_string()),
                            state.metadata.author.clone().unwrap_or_else(|| "未知作者".to_string())
                        )
                    };
                    
                    let mut state = app_state.write();
                    if !state.library.iter().any(|b| b.path == file_path) {
                        state.add_to_library(
                            file_path.clone(),
                            title,
                            author,
                            saved_chapter.unwrap_or(0)
                        );
                    }
                    
                    current_chapter.set(saved_chapter.unwrap_or(0));
                }
                Err(e) => load_error.set(Some(e.to_string())),
            }
        }
    });

    // 修改预计算章节数的方式
    let total_chapters = use_memo(move || {
        book_state.read().metadata.chapter_count
    });

    let go_next = move |_| {
        let new_chapter = *current_chapter.read() + 1;
        if new_chapter < book_state.read().metadata.chapter_count {
            goto_chapter(
                new_chapter
            );
        }
    };

    let go_prev = move |_| {

        let new_chapter = *current_chapter.read() - 1;
        if new_chapter > 0 {
            goto_chapter(
                new_chapter
            );
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
                TableOfContents { }
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
                    content_view{current_chapter}
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
                        disabled: *current_chapter.read() >= total_chapters.read().saturating_sub(1),
                        onclick: go_next,

                        "下一章"
                    }
                }
            }
        }
    }
}

#[component]
pub fn content_view(current_chapter: Signal<usize>) -> Element {
    let mut book_state = use_context::<Signal<BookState>>();
    let chapter_content = use_memo(move || {
        book_state.write().get_chapter(*current_chapter.read()).content
    });

    rsx! {
        div {
            class: "flex-1 p-8 overflow-y-auto bg-white text-gray-800 h-full relative",
            style: "z-index: 1",
            div {
                dangerous_inner_html: "{chapter_content}",
                style: "img {{ max-width: 100%; height: auto; display: block; margin: 1em auto; }}"
            }
        }
    }
}
