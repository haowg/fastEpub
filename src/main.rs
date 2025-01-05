#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus::document::Title;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::document::Stylesheet;

mod components;
use components::Header;
use components::EpubReader;

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
    let current_file = use_signal(|| String::new());

    rsx! {
        Title { "Smart Reader" }
        Stylesheet { href: asset!("src/assets/style.css") }
        Stylesheet { href: asset!("assets/mystyle.css") }
        div { 
            class: "flex flex-col h-screen",
            Header { current_file: current_file }
            EpubReader { current_file: current_file }
        }
    }
}