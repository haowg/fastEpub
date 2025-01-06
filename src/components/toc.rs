// toc.rs
use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;
use epub::doc::NavPoint;
use crate::components::epub_loader::BookState;

#[component]
fn TocItem(
    entry: NavPoint,
    depth: usize,
    parent_idx: usize,
    item_idx: usize,
    current_chapter: Signal<usize>,
    collapsed_nodes: Signal<HashSet<String>>,
    goto_chapter: EventHandler<usize>,
    toggle_collapse: EventHandler<String>,
) -> Element {
    let node_id = use_memo(move || format!("{}-{}", parent_idx, item_idx));
    let is_collapsed = collapsed_nodes.read().contains(&node_id());
    let has_children = !entry.children.is_empty();
    
    let class_name = if let Some(ci) = entry.play_order.checked_sub(1) {
        if ci == *current_chapter.read() {
            "flex-1 cursor-pointer text-left py-1 text-blue-700 font-bold"
        } else {
            "flex-1 cursor-pointer text-left py-1 text-gray-700 hover:text-gray-900 hover:bg-gray-300"
        }
    } else {
        "flex-1 cursor-pointer text-left py-1 text-gray-700 hover:text-gray-900 hover:bg-gray-300"
    };

    rsx! {
        div {
            key: "{node_id}",
            class: "flex flex-col",
            div {
                class: "flex items-center gap-1 hover:bg-gray-200 rounded",
                style: "padding-left: {depth * 16}px",
                
                // 折叠按钮或占位符
                div {
                    class: "w-4 h-4 flex items-center justify-center",
                    {if has_children {
                        rsx! {
                            button {
                                class: "text-gray-500 hover:text-gray-700 focus:outline-none",
                                onclick: move |_| toggle_collapse.call(node_id.read().clone()),
                                span {
                                    class: "transform transition-transform duration-200",
                                    style: if is_collapsed { "" } else { "transform: rotate(90deg)" },
                                    "▶"
                                }
                            }
                        }
                    } else {
                        rsx! { div { class: "w-4 h-4" } }
                    }}
                }
                
                div {
                    class: "{class_name}",
                    onclick: move |_| if let Some(ci) = entry.play_order.checked_sub(1) { 
                        goto_chapter.call(ci)
                    },
                    "{entry.label}"
                }
            }

            // 子节点
            {(!is_collapsed && has_children).then(|| rsx!(
                div {
                    class: "flex flex-col",
                    {entry.children.iter().enumerate().map(|(child_idx, child)| {
                        let child = child.clone();
                        rsx!(
                            TocItem {
                                key: "{node_id}-{child_idx}",
                                entry: child,
                                depth: depth + 1,
                                parent_idx: item_idx,
                                item_idx: child_idx,
                                current_chapter: current_chapter,
                                collapsed_nodes: collapsed_nodes,
                                goto_chapter: goto_chapter,
                                toggle_collapse: toggle_collapse,
                            }
                        )
                    })}
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
        div { 
            class: "flex flex-col gap-1 p-2 select-none",
            {book_state.read().toc.iter().enumerate().map(|(idx, entry)| {
                let entry = entry.clone();
                rsx!(
                    TocItem {
                        key: "{idx}",
                        entry: entry,
                        depth: 0,
                        parent_idx: 0,
                        item_idx: idx,
                        current_chapter: current_chapter,
                        collapsed_nodes: collapsed_nodes,
                        goto_chapter: goto_chapter,
                        toggle_collapse: move |id| toggle_collapse(id),
                    }
                )
            })}
        }
    }
}
