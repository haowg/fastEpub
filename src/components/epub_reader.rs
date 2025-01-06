use dioxus::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::components::{TableOfContents, BookMetadata, BookState, load_epub, AppState};

#[component]
pub fn EpubReader(
    current_file: Signal<String>,
    app_state: Signal<AppState>,
) -> Element {
    let mut book_state = use_signal(|| BookState {
        metadata: BookMetadata {
            title: None,
            author: None,
            description: None,
            cover_id: None,
            chapter_count: 0,
        },
        toc: Vec::new(),
        current_content: String::new(),
    });

    let mut current_chapter = use_signal(|| 0);
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

    use_effect(move || {
        let file_path = current.read();
        if (!file_path.is_empty() && file_path.ends_with(".epub")) {
            // 修复字符串比较
            let (saved_chapter, existing_book) = {
                let state = app_state.read();
                (
                    state.get_progress(&file_path),
                    state.library.iter().find(|b| *b.path == *file_path).cloned()
                )
            };
            
            match load_epub(&file_path, book_state.clone()) {
                Ok(_) => {
                    load_error.set(None);
                    
                    // 仅在书籍不存在时添加到书库
                    if existing_book.is_none() {
                        let (title, author) = {
                            let state = book_state.read();
                            (
                                state.metadata.title.clone().unwrap_or_else(|| "未知标题".to_string()),
                                state.metadata.author.clone().unwrap_or_else(|| "未知作者".to_string())
                            )
                        };
                        
                        let mut state = app_state.write();
                        state.add_to_library(
                            file_path.clone(),
                            title,
                            author,
                            saved_chapter.unwrap_or(0)
                        );
                    }
                    
                    // 设置章节位置
                    current_chapter.set(saved_chapter.unwrap_or(0));
                }
                Err(e) => load_error.set(Some(e.to_string())),
            }
        }
        
        // 返回清理函数
        ()
    });

    use_effect(move || {
        let chapter = *current_chapter.read();
        let file_path = current.read();
        
        if !file_path.is_empty() && file_path.ends_with(".epub") {
            if let Ok(mut doc) = epub::doc::EpubDoc::new(&*file_path) {
                let chapter = BookState::get_chapter(&mut doc, chapter);
                book_state.write().current_content = chapter.content;
            }
        }
        
        ()
    });

    // 使用缓存内容而不是直接读取
    let chapter_content = use_memo(move || {
        let state = book_state.read();
        state.current_content.clone()
    });

    // 修改预计算章节数的方式
    let total_chapters = use_memo(move || {
        book_state.read().metadata.chapter_count
    });

    let go_next = move |_| {
        let mut state = current_chapter.write();
        if (*state + 1 < *total_chapters.read()) {
            *state += 1;
            let mut app = app_state.write();
            app.update_progress(current_file.read().to_string(), *state);
        }
    };

    let go_prev = move |_| {
        let mut state = current_chapter.write();
        if (*state > 0) {
            *state -= 1;
            let mut app = app_state.write();
            app.update_progress(current_file.read().to_string(), *state);
        }
    };

    let mut goto_chapter = move |idx: usize| {
        let mut state = current_chapter.write();
        if (idx < *total_chapters.read()) {
            *state = idx;
            // 保存阅读进度
            let mut app = app_state.write();
            app.update_progress(current_file.read().to_string(), idx);
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
                    current_chapter: current_chapter,
                    book_state: book_state,
                    goto_chapter: move |idx| goto_chapter(idx),
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
                        dangerous_inner_html: "{chapter_content.read()}"
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
                        disabled: *current_chapter.read() >= total_chapters.read().saturating_sub(1),
                        onclick: go_next,
                        "下一章"
                    }
                }
            }
        }
    }
}
