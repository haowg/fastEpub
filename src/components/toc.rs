use dioxus::prelude::*;
use std::collections::HashSet;
use std::println;
use epub::doc::NavPoint;
use crate::components::epub_loader::BookState;

#[derive(Props, PartialEq, Clone)]
pub struct TableOfContentsProps {
    on_select: EventHandler<usize>,
}

#[derive(Props, PartialEq, Clone)]
pub struct TocItemProps {
    entry: NavPoint,
    depth: usize,
    parent_idx: usize,
    item_idx: usize,
    collapsed_nodes: Signal<HashSet<String>>,
    on_toggle: EventHandler<String>,
    on_select: EventHandler<usize>,
}

// 新增一个扁平化的目录项结构，用于优化渲染
#[derive(Clone, Debug, PartialEq)]
struct FlatTocItem {
    label: String,
    play_order: usize,
    path: String,
    depth: usize,
    has_children: bool,
    node_id: String,
}

#[component(memoize = true)]
fn TocItem(props: TocItemProps) -> Element {
    let current_chapter = use_context::<Signal<usize>>();
    let node_id = format!("{}-{}", props.parent_idx, props.item_idx);
    let has_children = !props.entry.children.is_empty();

    // 使用 use_memo 缓存折叠状态，减少不必要的计算
    let node_id_for_memo = node_id.clone();
    let is_collapsed = use_memo(move || {
        props.collapsed_nodes.read().contains(&node_id_for_memo)
    });
    
    let chapter_index = props.entry.play_order;
    
    // 缓存当前选中状态的样式
    let class_name = use_memo(move || {
        if props.entry.play_order == *current_chapter.read() {
            "flex-1 cursor-pointer text-left py-1 text-blue-700 font-bold"
        } else {
            "flex-1 cursor-pointer text-left py-1 text-gray-700 hover:text-gray-900 hover:bg-gray-300"
        }
    });

    // 避免在渲染时创建闭包，预先构建
    let toggle_id = node_id.clone();
    let toggle_handler = move |_| props.on_toggle.call(toggle_id.clone());
    
    let select_handler = move |_| props.on_select.call(props.entry.play_order);

    rsx! {
        div {
            class: "flex flex-col",
            div {
                class: "flex items-center gap-1 hover:bg-gray-200 rounded",
                style: "padding-left: {props.depth * 16}px",
                
                // 折叠按钮
                div {
                    class: "w-4 h-4 flex items-center justify-center",
                    {if has_children {
                        rsx! {
                            button {
                                class: "text-gray-500 hover:text-gray-700 focus:outline-none",
                                onclick: toggle_handler,
                                span {
                                    class: "transform transition-transform duration-200",
                                    if *is_collapsed.read() { "▶" } else { "▼" },
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
                
                // 标签
                div {
                    class: "{class_name}",
                    onclick: select_handler,
                    "{props.entry.label}"
                }
            }

            // 子目录 - 只在需要时渲染
            {(!*is_collapsed.read() && has_children).then(|| rsx!(
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

#[derive(Props, PartialEq, Clone)]
pub struct TocChildrenProps {
    pub nav_points: Vec<NavPoint>,
    pub depth: usize,
    pub parent_idx: usize,
    pub on_toggle: EventHandler<String>,
    pub on_select: EventHandler<usize>,
    pub collapsed_nodes: Signal<HashSet<String>>,
}

#[component(memoize = true)]
fn TocChildren(props: TocChildrenProps) -> Element {
    // 使用 use_memo 预处理子项，避免在渲染循环中频繁计算
    let children_items = use_memo(move || {
        props.nav_points.iter().enumerate().map(|(idx, child)| {
            let node_id = format!("{}-{}", props.parent_idx, idx);
            (idx, node_id, child.clone())
        }).collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: "flex flex-col",
            {children_items.read().iter().map(|(idx, node_id, child)| {
                rsx!(
                    TocItem {
                        key: "{node_id}",
                        entry: child.clone(),
                        depth: props.depth,
                        parent_idx: props.parent_idx,
                        item_idx: *idx,
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
pub fn TableOfContents(props: TableOfContentsProps) -> Element {
    let book_state = use_context::<Signal<BookState>>();
    let mut collapsed_nodes = use_signal(|| HashSet::new());
    
    // 优化：缓存目录数据，只有在书籍变化时才重新计算
    let toc_data = use_memo(move || book_state.read().toc.clone());
    
    // 预先转换顶级项
    let root_items = use_memo(move || {
        toc_data.read().iter().enumerate().map(|(idx, entry)| {
            let node_id = format!("0-{}", idx);
            (idx, node_id, entry.clone())
        }).collect::<Vec<_>>()
    });

    // 优化：避免每次渲染时创建闭包
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
            
            // 显示目录项 - 移除滚动设置
            div {
                // 删除 overflow-auto 和 max-height 限制
                {root_items.read().iter().map(|(idx, node_id, entry)| {
                    rsx!(
                        TocItem {
                            key: "{node_id}",
                            entry: entry.clone(),
                            depth: 0,
                            parent_idx: 0,
                            item_idx: *idx,
                            collapsed_nodes: collapsed_nodes.clone(),
                            on_toggle: move |id| on_toggle(id),
                            on_select: props.on_select.clone(),
                        }
                    )
                })}
            }
        }
    }
}
