use dioxus::prelude::*;
use dioxus::desktop::window;
use rfd::FileDialog;
use crate::components::{AppState, MenuButton};
use std::collections::HashMap;
use std::time::Instant;

#[component]
pub fn Header(
    current_file: Signal<String>,
    show_library: Signal<bool>,
    app_state: Signal<AppState>,
) -> Element {
    let mut maximized = use_signal(|| window().is_maximized());
    let mut is_dragging = use_signal(|| false);
    let mut mouse_down_pos = use_signal(|| None::<(f64, f64)>);

    use_effect(move || {
        if window().is_maximized() != *maximized.read() {
            maximized.set(window().is_maximized());
        }
        ()
    });

    rsx! {
        header {
            class: "text-gray-400 bg-gray-900 body-font relative",

            div {
                class: "flex p-2 flex-row items-center justify-between w-full",
                // Left: Menu button
                div {
                    class: "w-24",
                    MenuButton {
                        show_library: show_library,
                        app_state: app_state,
                        current_file: current_file,
                    }
                }

                // Center: Title
                div {

                    class: "flex-1 flex title-font font-medium items-center justify-center text-white select-none cursor-default",
                    "data-tauri-drag-region": "",
                    onmousedown: move |evt| {
                        let coords = evt.client_coordinates();
                        mouse_down_pos.set(Some((coords.x, coords.y)));
                        is_dragging.set(false);
                    },
                    onmousemove: move |evt| {
                        if let Some((start_x, start_y)) = *mouse_down_pos.read() {
                            let coords = evt.client_coordinates();
                            if (coords.x - start_x).abs() > 5.0 || (coords.y - start_y).abs() > 5.0 {
                                is_dragging.set(true);
                                window().drag();
                            }
                        }
                    },
                    onmouseup: move |evt| {
                        mouse_down_pos.set(None);
                        is_dragging.set(false);
                    },
                    ondoubleclick: move |evt| {
                        evt.stop_propagation();
                        let will_maximize = !window().is_maximized();
                        window().set_maximized(will_maximize);
                        maximized.set(will_maximize);
                    },
                    span {
                        class: "text-xl select-none pointer-events-none",
                        "FastEpub"
                    },
                }

                // Right: Window controls
                div {
                    class: "w-24 flex justify-end items-center space-x-2 mr-2",
                    button {
                        class: "w-4 h-4 flex items-center justify-center bg-yellow-500 hover:bg-yellow-600 text-white rounded-full text-xs leading-none",
                        onmousedown: |evt| evt.stop_propagation(),
                        onclick: move |_| window().set_minimized(true),
                        "─"
                    }
                    button {
                        class: "w-4 h-4 flex items-center justify-center bg-green-500 hover:bg-green-600 text-white rounded-full text-xs leading-none",
                        onmousedown: |evt| evt.stop_propagation(),
                        onclick: move |_| {
                            let will_maximize = !window().is_maximized();
                            window().set_maximized(will_maximize);
                            maximized.set(will_maximize);
                        },
                        {if *maximized.read() {
                            "□"
                        } else {
                            "∧"
                        }}
                    }
                    button {
                        class: "w-4 h-4 flex items-center justify-center bg-red-500 hover:bg-red-600 text-white rounded-full text-xs leading-none",
                        onmousedown: |evt| evt.stop_propagation(),
                        onclick: move |_| window().close(),
                        "✕"
                    }
                }
            }
        }
    }
}

