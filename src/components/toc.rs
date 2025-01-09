use dioxus::prelude::*;
use std::collections::HashSet;
use std::println;
use epub::doc::NavPoint;
use crate::components::epub_loader::BookState;
use crate::components::epub_reader::goto_chapter;

#[derive(Props, PartialEq, Clone)]
pub struct TocItemProps {
    entry: NavPoint,
    depth: usize,
    parent_idx: usize,
    item_idx: usize,
    collapsed_nodes: Signal<HashSet<String>>, // replace is_collapsed: bool with the signal
    on_toggle: EventHandler<String>,
    on_select: EventHandler<usize>,
}

#[derive(Props, PartialEq, Clone)]
pub struct TocChildrenProps {
    pub nav_points: Vec<NavPoint>, // renamed from "children"
    pub depth: usize,
    pub parent_idx: usize,
    pub on_toggle: EventHandler<String>,
    pub on_select: EventHandler<usize>,
    pub collapsed_nodes: Signal<HashSet<String>>, // replace bool with the signal
}

#[component(memoize = true)]
fn TocItem(props: TocItemProps) -> Element {
    let current_chapter = use_context::<Signal<usize>>();
    let node_id = format!("{}-{}", props.parent_idx, props.item_idx);
    let has_children = !props.entry.children.is_empty();
    let is_collapsed = props.collapsed_nodes.read().contains(&node_id);
    let chapter_index = props.entry.play_order.saturating_sub(1);
    
    let class_name = use_memo(move || {
        if props.entry.play_order == *current_chapter.read() {
            "flex-1 cursor-pointer text-left py-1 text-blue-700 font-bold"
        } else {
            "flex-1 cursor-pointer text-left py-1 text-gray-700 hover:text-gray-900 hover:bg-gray-300"
        }
    });

    rsx! {
        div {
            class: "flex flex-col",
            div {
                class: "flex items-center gap-1 hover:bg-gray-200 rounded",
                style: "padding-left: {props.depth * 16}px",
                
                // Collapse button
                div {
                    class: "w-4 h-4 flex items-center justify-center",
                    {if has_children {
                        rsx! {
                            button {
                                class: "text-gray-500 hover:text-gray-700 focus:outline-none",
                                onclick: move |_| props.on_toggle.call(node_id.clone()),
                                span {
                                    class: "transform transition-transform duration-200",
                                    style: if is_collapsed { "" } else { "transform: rotate(90deg)" },
                                    "▶"
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                class: "w-4 h-4"
                            }
                        }
                    }}
                }
                
                // Label
                div {
                    class: "{class_name}",
                    onclick: move |_| { 
                        props.on_select.call(props.entry.play_order);
                    },
                    "{props.entry.label}"
                }
            }

            // Children
            {(!is_collapsed && has_children).then(|| rsx!(
                TocChildren {
                    nav_points: props.entry.children.clone(),
                    depth: props.depth + 1,
                    parent_idx: props.item_idx,
                    on_toggle: props.on_toggle.clone(),
                    on_select: props.on_select.clone(),
                    collapsed_nodes: props.collapsed_nodes.clone(),
                }
            ))}
        }
    }
}

#[component(memoize = true)]
fn TocChildren(props: TocChildrenProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col",
            {props.nav_points.iter().enumerate().map(|(idx, child)| {
                let node_id = format!("{}-{}", props.parent_idx, idx);
                rsx!(
                    TocItem {
                        key: "{node_id}",
                        entry: child.clone(),
                        depth: props.depth,
                        parent_idx: props.parent_idx,
                        item_idx: idx,
                        collapsed_nodes: props.collapsed_nodes.clone(),
                        on_toggle: props.on_toggle.clone(),
                        on_select: props.on_select.clone(),
                    }
                )
            })}
        }
    }
}

#[component]
pub fn TableOfContents() -> Element {
    let book_state = use_context::<Signal<BookState>>();
    let mut collapsed_nodes = use_signal(|| HashSet::new());
    
    let toc_data = use_memo(move || book_state.read().toc.clone());

    let mut on_toggle = move |id: String| {
        let mut nodes = collapsed_nodes.write();
        if nodes.contains(&id) {
            nodes.remove(&id);
        } else {
            nodes.insert(id);
        }
    };

    rsx! {
        div { 
            class: "flex flex-col gap-1 p-2 select-none",
            {toc_data.read().iter().enumerate().map(|(idx, entry)| {
                let node_id = format!("0-{}", idx);
                rsx!(
                    TocItem {
                        key: "{node_id}",
                        entry: entry.clone(),
                        depth: 0,
                        parent_idx: 0,
                        item_idx: idx,
                        collapsed_nodes: collapsed_nodes.clone(), // pass entire signal
                        on_toggle: move |id| on_toggle(id),
                        on_select: move |chapter| goto_chapter(chapter),
                    }
                )
            })}
        }
    }
}
