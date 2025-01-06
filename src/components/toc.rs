// toc.rs
use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;
use epub::doc::NavPoint;
use crate::components::epub_loader::BookState;

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

            // 子节点
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
pub fn TableOfContents(
    current_chapter: Signal<usize>,
    book_state: Signal<BookState>,
    goto_chapter: EventHandler<usize>,
) -> Element {
    let mut collapsed_nodes = use_signal(|| HashSet::new());
    
    let mut toggle_collapse = move |idx: String| {
        let mut nodes = collapsed_nodes.write();
        if nodes.contains(&idx) {
            nodes.remove(&idx);
        } else {
            nodes.insert(idx);
        }
    };

    rsx! {
        div { class: "flex flex-col gap-1 p-2",
            {book_state.read().toc.iter().map(|entry| rsx!(
                TocItem {
                    key: "{entry.label}",
                    entry: entry.clone(),
                    depth: 0,
                    current_chapter: current_chapter,
                    collapsed_nodes: collapsed_nodes,
                    book_state: book_state,
                    goto_chapter: goto_chapter,
                    toggle_collapse: move |id| toggle_collapse(id),
                }
            ))}
        }
    }
}
