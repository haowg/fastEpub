#![allow(non_snake_case)]
#![cfg_attr(windows, windows_subsystem = "windows")]
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
    let app_state = use_context_provider(|| Signal::new(AppState::load()));
    let current_file = use_context_provider(|| {
        Signal::new(
            app_state.read().get_last_book()
                .map(|(path, _)| path)
                .unwrap_or_default()
        )
    });
    let show_library = use_signal(|| false);

    rsx! {
        Title { "Fast Epub" }
        Stylesheet { href: asset!("src/assets/style.css") }
        Stylesheet { href: asset!("assets/mystyle.css") }
        div { 
            class: "flex flex-col h-screen",
            Header { 
                show_library: show_library,
            }
            div {
                class: "flex-1 relative",
                if *show_library.read() {
                    Library {
                        show_library: show_library,
                    }
                } else {
                    EpubReader {}
                }
            }
        }
    }
}