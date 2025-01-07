use dioxus::prelude::*;
use rfd::FileDialog;
use crate::components::AppState;

#[component]
pub fn MenuButton(
    show_library: Signal<bool>,
    app_state: Signal<AppState>,
    current_file: Signal<String>,
) -> Element {
    let mut dropdown_open = use_signal(|| false);

    let close_dropdown = move |_| {
        if *dropdown_open.read() {
            dropdown_open.set(false);
        }
    };

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
                show_library.set(false);
            }
        }
        dropdown_open.set(false);
    };

    let toggle_theme = move |_| {
        dropdown_open.set(false);
    };

    rsx! {
        div {
            class: "relative",
            // 遮罩层
            {if *dropdown_open.read() {
                rsx! {
                    div {
                        class: "fixed inset-0 h-full w-full z-30",
                        onclick: close_dropdown,
                    }
                }
            } else {
                rsx! { div {} }
            }
        }

            // 菜单按钮
            button {
                class: "inline-flex items-center bg-gray-700 hover:bg-gray-600 text-white rounded-lg px-3 py-1",
                onclick: move |evt| {
                    evt.stop_propagation();
                    dropdown_open.toggle();
                },
                span { "☰" }
                span { class: "ml-1", "菜单" }
            }
            
            // 下拉菜单
            {if *dropdown_open.read() {
                rsx! {
                    div {
                        class: "absolute left-0 mt-2 w-56 bg-white rounded-lg shadow-lg z-40",
                        onclick: move |evt| evt.stop_propagation(),
                        
                        div { class: "px-2 py-2 border-b border-gray-200",
                            button {
                                class: "w-full text-left px-4 py-2 text-gray-800 hover:bg-gray-100 rounded-lg flex items-center",
                                onclick: open_file,
                                span { class: "mr-2", "📂" }
                                "打开文件"
                            }
                        }
                        
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
                                onclick: |evt| evt.stop_propagation(),
                                span { class: "mr-2", "⚙️" }
                                "阅读设置"
                            }
                        }
                    }
                }
            } else {
                rsx! { div {} }
            }
        }
        }
    }
}
