use dioxus::prelude::*;
use dioxus::desktop::window;
use rfd::FileDialog;
use crate::components::AppState;
use std::collections::HashMap;

#[component]
pub fn Header(
    current_file: Signal<String>,
    show_library: Signal<bool>,
    app_state: Signal<AppState>,  // 添加 app_state
) -> Element {
    let mut fullscreen = use_signal(|| false);
    let mut dropdown_open = use_signal(|| false);

    // 添加点击外部关闭下拉菜单的处理
    let close_dropdown = move |_| {
        if *dropdown_open.read() {
            dropdown_open.set(false);
        }
    };

    // 打开文件处理函数
    let open_file = move |_| {
        if let Some(file) = FileDialog::new()
            .add_filter("EPUB", &["epub"])
            .set_directory("/")
            .pick_file() 
        {
            if let Some(path) = file.to_str() {
                let mut app_state = AppState::load();
                app_state.last_book = Some(path.to_string());
                let _ = app_state.save();
                current_file.set(path.to_string());
                show_library.set(false); // 添加这行以关闭书库页
            }
        }
        dropdown_open.set(false);
    };

    // 切换主题处理函数
    let toggle_theme = move |_| {
        // TODO: 实现主题切换逻辑
        dropdown_open.set(false);
    };

    rsx! {
        header {
            class: "text-gray-400 bg-gray-900 body-font relative", // 添加 relative 定位

            // 添加遮罩层（放在最前面）
            {if *dropdown_open.read() {
                rsx! {
                    div {
                        class: "fixed inset-0 h-full w-full z-30",
                        onclick: close_dropdown,
                    }
                }
            } else {
                rsx! { Fragment {} }
            }}

            div {
                class: "flex p-2 flex-row items-center justify-between w-full",
                // Left: Menu button
                div {
                    class: "w-24", // 固定宽度，与右侧按钮组对称
                    // 下拉菜单容器
                    div {
                        class: "relative",
                        button {
                            class: "inline-flex items-center bg-gray-700 hover:bg-gray-600 text-white rounded-lg px-3 py-1",
                            onclick: move |evt| {
                                evt.stop_propagation();
                                dropdown_open.toggle();
                            },
                            span { "☰" }
                            span { class: "ml-1", "菜单" }
                        }
                        
                        // 下拉菜单内容
                        {if *dropdown_open.read() {
                            rsx! {
                                div {
                                    class: "absolute left-0 mt-2 w-56 bg-white rounded-lg shadow-lg z-40",
                                    onclick: move |evt| evt.stop_propagation(),
                                    
                                    // 文件操作菜单组
                                    div { class: "px-2 py-2 border-b border-gray-200",
                                        button {
                                            class: "w-full text-left px-4 py-2 text-gray-800 hover:bg-gray-100 rounded-lg flex items-center",
                                            onclick: open_file,
                                            span { class: "mr-2", "📂" }
                                            "打开文件"
                                        }
                                    }
                                    
                                    // 设置菜单组
                                    div { class: "px-2 py-2 border-b border-gray-200",
                                        button {
                                            class: "w-full text-left px-4 py-2 text-gray-800 hover:bg-gray-100 rounded-lg flex items-center",
                                            onclick: move |_| show_library.set(true),
                                            span { class: "mr-2", "📚" }
                                            "我的书库"
                                        }
                                    }
                                    div { class: "px-2 py-2",
                                        button {
                                            class: "w-full text-left px-4 py-2 text-gray-800 hover:bg-gray-100 rounded-lg flex items-center",
                                            onclick: toggle_theme,
                                            span { class: "mr-2", "🎨" }
                                            "切换主题"
                                        }
                                        button {
                                            class: "w-full text-left px-4 py-2 text-gray-800 hover:bg-gray-100 rounded-lg flex items-center",
                                            onclick: |evt| {
                                                evt.stop_propagation();
                                                // TODO: 实现设置功能
                                            },
                                            span { class: "mr-2", "⚙️" }
                                            "阅读设置"
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! { Fragment {} }
                        }
                    }
                    }
                }

                // Center: Title - 修改此处
                a {
                    class: "flex-1 flex title-font font-medium items-center justify-center text-white select-none",
                    onmousedown: move |_| window().drag(),
                    span {
                        class: "text-xl",  // 移除 ml-3，避免偏移
                        "FastEpub"
                    }
                }

                // Right: Window controls
                div {
                    class: "w-24 flex justify-end space-x-2", // 固定宽度
                    button {
                        /* 最小化按钮，黄颜色圆形样式 */
                        class: "w-4 h-4 flex items-center justify-center bg-yellow-500 hover:bg-yellow-600 text-white rounded-full text-xs",
                        onmousedown: |evt| evt.stop_propagation(),
                        onclick: move |_| window().set_minimized(true),
                        "-"
                    }
                    button {
                        /* 全屏按钮，绿颜色圆形样式 */
                        class: "w-4 h-4 flex items-center justify-center bg-green-500 hover:bg-green-600 text-white rounded-full text-xs",
                        onmousedown: |evt| evt.stop_propagation(),
                        onclick: move |_| {
                            window().set_fullscreen(!fullscreen());
                            fullscreen.toggle();
                        },
                        "□"
                    }
                    button {
                        /* 关闭按钮，红颜色圆形样式 */
                        class: "w-4 h-4 flex items-center justify-center bg-red-500 hover:bg-red-600 text-white rounded-full text-xs",
                        onmousedown: |evt| evt.stop_propagation(),
                        onclick: move |_| window().close(),
                        "╳"
                    }
                }
            }
        }
    }
}
