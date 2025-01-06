use dioxus::prelude::*;
use crate::components::{AppState, BookInfo};
use std::cmp::PartialEq;
use std::path::PathBuf;

// 添加单独的卡片组件
#[component]
fn BookCard(
    book: BookInfo,
    current_file: Signal<String>,
    show_library: Signal<bool>,
) -> Element {
    let file_exists = PathBuf::from(&book.path).exists();
    let card_class = if file_exists {
        "bg-white dark:bg-gray-800 rounded-lg transition-all cursor-pointer hover:shadow-lg hover:-translate-y-1 flex flex-col w-40 h-56"  // 固定卡片尺寸
    } else {
        "bg-white dark:bg-gray-800 rounded-lg transition-all opacity-50 flex flex-col w-40 h-56"
    };
    let last_read = book.last_read.format("%m-%d %H:%M");
    
    rsx! {
        div { 
            key: "{book.path}",
            class: "{card_class} relative m-0",  // 添加 relative 和 margin-0
            style: "max-width: 160px",  // 固定最大宽度
            onclick: move |_| if file_exists {
                current_file.set(book.path.clone());
                show_library.set(false);
            },
            // "书脊" 装饰
            div {
                class: "absolute left-0 top-0 bottom-0 w-1 bg-blue-500 rounded-l-lg"
            }
            // 封面区域
            div {
                class: "h-32 bg-gray-100 dark:bg-gray-700 rounded-t-lg flex items-center justify-center",
                span { 
                    class: "text-4xl text-gray-400 dark:text-gray-500",
                    "📚" 
                }
            }
            // 信息区域
            div {
                class: "p-3 flex-1 flex flex-col min-h-0",
                div { 
                    class: "font-bold text-sm text-gray-800 dark:text-white line-clamp-2",
                    title: "{book.title}",
                    "{book.title}" 
                }
                div { 
                    class: "text-gray-600 dark:text-gray-400 text-xs truncate mt-1",
                    title: "{book.author}",
                    "{book.author}" 
                }
                div { 
                    class: "mt-auto pt-1 space-y-0.5 text-[10px]",
                    div {
                        class: "text-gray-500 dark:text-gray-500 truncate",
                        "阅读于 {last_read}"
                    }
                    div {
                        class: "text-blue-500 dark:text-blue-400 truncate",
                        "第 {book.chapter_index + 1} 章"
                    }
                }
            }
        }
    }
}

#[component]
pub fn Library(
    current_file: Signal<String>,
    show_library: Signal<bool>,
    app_state: Signal<AppState>,
) -> Element {
    // 缓存当前文件
    let current = use_memo(move || current_file.read().to_string());
    
    // 缓存书籍列表，只在 app_state 变化时更新
    let books = use_memo(move || {
        let state = app_state.read();
        let mut books = state.get_library();
        books.retain(|book| PathBuf::from(&book.path).exists());
        books
    });

    // 仅在组件挂载和当前文件变化时更新进度
    use_effect(move || {
        if !current().is_empty() {
            // 一次性读取所需的所有数据
            let (progress, needs_update) = {
                let state = app_state.read();
                let progress = state.reading_progress.get(&current()).cloned();
                let needs_update = state.library.iter()
                    .find(|b| b.path == current())
                    .map(|book| {
                        progress.as_ref().map(|p| {
                            p.chapter_index != book.chapter_index || 
                            p.last_read != book.last_read
                        }).unwrap_or(false)
                    })
                    .unwrap_or(false);
                (progress, needs_update)
            };
            
            // 只在确实需要更新时才写入状态
            if needs_update {
                if let Some(progress) = progress {
                    let mut state = app_state.write();
                    if let Some(book) = state.library.iter_mut().find(|b| b.path == current()) {
                        book.chapter_index = progress.chapter_index;
                        book.last_read = progress.last_read;
                        state.save().ok();
                    }
                }
            }
        }
        
        ()
    });

    // 渲染空状态组件
    let render_empty_state = move || {
        rsx! {
            div {
                class: "flex flex-col items-center justify-center h-[calc(100vh-64px)] text-gray-500 dark:text-gray-400",
                span { class: "text-6xl mb-4", "📚" }
                p { class: "text-lg", "还没有添加任何书籍" }
            }
        }
    };

    // 渲染书籍网格
    let render_book_grid = move |books: &[BookInfo]| {
        rsx! {
            div { 
                class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6 justify-items-center",  // 使用响应式网格列数
                {books.iter().map(|book| rsx!(
                    BookCard {
                        book: book.clone(),
                        current_file: current_file,
                        show_library: show_library,
                    }
                ))}
            }
        }
    };

    rsx! {
        div { 
            class: "absolute inset-0 bg-gray-100 dark:bg-gray-900 overflow-auto",
            
            // Header
            div { 
                class: "sticky top-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4 flex justify-between items-center shadow-sm",
                h1 { class: "text-2xl font-bold text-gray-800 dark:text-white", "我的书库" }
                div { 
                    class: "flex items-center gap-2",
                    span { class: "text-sm text-gray-500 dark:text-gray-400", "{books.len()} 本书" }
                    button { 
                        class: "p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-full text-gray-500 dark:text-gray-400",
                        onclick: move |_| show_library.set(false),
                        "✕"
                    }
                }
            }
            
            // Content
            div { 
                class: "container mx-auto px-8 py-8",  // 增加内边距
                {if books.is_empty() {
                    render_empty_state()
                } else {
                    render_book_grid(&books())
                }}
            }
        }
    }
}
