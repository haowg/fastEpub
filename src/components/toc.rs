// toc.rs
use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;
use epub::doc::NavPoint;
use crate::components::epub_loader::{BookState};
use crate::components::{AppState, goto_chapter};

fn toggle_node_collapse(nodes: &mut HashSet<String>, idx: String) {
    if nodes.contains(&idx) {
        nodes.remove(&idx);
    } else {
        nodes.insert(idx);
    }
}

#[component]
fn TocItem(
    entry: NavPoint,
    depth: usize,
    parent_idx: usize,
    item_idx: usize,
    collapsed_nodes: Signal<HashSet<String>>,
    toggle_collapse: EventHandler<String>,
) -> Element {
    let current_chapter = use_context::<Signal<usize>>();
    let node_id = use_memo(move || format!("{}-{}", parent_idx, item_idx));
    let is_collapsed = collapsed_nodes.read().contains(&node_id());
    let has_children = !entry.children.is_empty();
    // println!("{}: {}", node_id(), is_collapsed);
    
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
                        goto_chapter(ci);
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
                                collapsed_nodes: collapsed_nodes,
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
) -> Element {
    let book_state = use_context::<Signal<BookState>>();
    println!("TableOfContents");
    
    // 使用 memo 缓存目录数据，避免不必要的重渲染
    let toc_data = use_memo(move || {
        book_state.read().toc.clone()
    });

    // 缓存折叠状态
    let mut collapsed_nodes = use_signal(|| HashSet::new());
    
    rsx! {
        div { 
            class: "flex flex-col gap-1 p-2 select-none",
            {toc_data.read().iter().enumerate().map(|(idx, entry)| {
                let entry = entry.clone();
                rsx!(
                    TocItem {
                        key: "{idx}",
                        entry: entry,
                        depth: 0,
                        parent_idx: 0,
                        item_idx: idx,
                        collapsed_nodes: collapsed_nodes,
                        toggle_collapse: move |id| toggle_node_collapse(&mut collapsed_nodes.write(), id),
                    }
                )
            })}
        }
    }
}
