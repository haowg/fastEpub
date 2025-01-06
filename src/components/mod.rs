mod header;
mod epub_reader;
mod toc;
mod epub_loader;
mod html_processor;
mod storage;
mod library;

pub use header::Header;
pub use epub_reader::EpubReader;
pub(crate) use toc::TableOfContents;
pub(crate) use epub_loader::{BookState, Chapter, BookMetadata, load_epub};
pub(crate) use html_processor::process_html_content;
pub(crate) use storage::{AppState, BookInfo};
pub(crate) use library::Library;
