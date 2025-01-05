# Smart Reader

<div align="center">

![Smart Reader Logo](assets/logo.png)

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)
[![Dioxus](https://img.shields.io/badge/dioxus-latest-blue.svg)](https://dioxuslabs.com)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A modern, cross-platform EPUB reader built with Rust and Dioxus.

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Development](#development) • [Contributing](#contributing)

</div>

## Features

- ⚡️ **Lightning Fast**: Instant file loading and chapter switching with Rust's zero-cost abstractions
- 🚀 **Memory Efficient**: Optimized memory usage for large EPUB files
- 📚 **Full EPUB Support**: Read EPUB 2.0 and 3.0 files
- 📑 **Interactive TOC**: Easy navigation with interactive table of contents
- 🎨 **Modern UI**: Clean, responsive interface with custom window decorations
- 🌙 **Theme Support**: Light and dark mode (coming soon)
- 🔖 **Bookmarks**: Save and manage your reading progress (coming soon)
- 💾 **Persistent Settings**: Remember your preferences across sessions
- ↔️ **Customizable Layout**: Adjustable sidebar and reading pane
- 🖥️ **Cross-Platform**: Works on Windows, macOS, and Linux

## Performance

Smart Reader is built with performance in mind:

- **Fast Loading**: Opens large EPUB files (100MB+) in milliseconds
- **Efficient Memory**: Loads chapters on-demand to minimize memory usage
- **Smooth Navigation**: Instantly switches between chapters with no lag
- **Native Performance**: Built with Rust for maximum efficiency
- **Optimized Parsing**: Uses high-performance EPUB parsing engine

## Installation

### Prerequisites

- Rust (stable) - [rustup.rs](https://rustup.rs)
- Node.js and npm - [nodejs.org](https://nodejs.org)
- Git

### Build from Source

1. Clone the repository:

### Tailwind
1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the Tailwind CSS CLI: https://tailwindcss.com/docs/installation
3. Run the following command in the root of the project to start the Tailwind CSS compiler:

```bash
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```
````

