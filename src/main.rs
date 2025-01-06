#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus::document::Title;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::document::Stylesheet;

mod components;
use components::{Header, EpubReader, AppState, Library};

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(
            WindowBuilder::new()
                .with_resizable(true)
                .with_decorations(false) // Disable native window decorations
                .with_title("Smart Reader")
        ))
        .launch(App)
}

#[component]
fn App() -> Element {
    let app_state = use_signal(AppState::load);
    let current_file = use_signal(|| {
        app_state.read().get_last_book()
            .map(|(path, _)| path)
            .unwrap_or_default()
    });
    let show_library = use_signal(|| false);

    rsx! {
        Title { "Fast Epub" }
        Stylesheet { href: asset!("src/assets/style.css") }
        Stylesheet { href: asset!("assets/mystyle.css") }
        div { 
            class: "flex flex-col h-screen",
            Header { 
                current_file: current_file,
                show_library: show_library,
                app_state: app_state,
            }
            // 内容区域容器
            div {
                class: "flex-1 relative", // 添加 relative 以便定位子元素
                // 阅读器和书库在同一层，通过条件显示控制
                if *show_library.read() {
                    Library {
                        current_file: current_file,
                        show_library: show_library,
                        app_state: app_state,
                    }
                } else {
                    EpubReader { 
                        current_file: current_file,
                        app_state: app_state,
                    }
                }
            }
        }
    }
}