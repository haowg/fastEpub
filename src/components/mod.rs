mod header;
mod menu;
mod epub_reader;
mod toc;
mod epub_loader;
mod html_processor;
mod storage;
mod library;

pub use header::Header;
pub use epub_reader::{EpubReader, goto_chapter}; // 更新导出
pub(crate) use menu::MenuButton;
pub(crate) use toc::TableOfContents;
pub(crate) use epub_loader::{BookState, Chapter, BookMetadata, load_epub};
pub(crate) use html_processor::process_html_content;
pub(crate) use storage::{AppState, BookInfo};
pub(crate) use library::Library;
