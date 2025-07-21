# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Test
```bash
# Build the library
cargo build

# Build with examples
cargo build --examples

# Run examples
cargo run --example demo
cargo run --example demo_tabs

# Check code
cargo check
cargo clippy

# Format code
cargo fmt

# Run tests
cargo test
```

## Project Architecture

This is a Rust library (`egui_collapsible_dock`) that provides collapsible dock panel functionality for egui applications.

### Core Components

- **`src/dock_collapsible.rs`**: Main implementation containing the `CollapsibleDockPanel` struct and related types
  - `CollapsibleDockPanel<Tab>`: The primary component that integrates with `egui_dock` to provide collapsible panels
  - `CollapsibleDockState`: Manages panel state including collapse status, sizes, and persistence
  - `CollapsibleButton`: Button configuration for collapsed state
  - `PanelSide`: Enum for panel positioning (Left, Right, Top, Bottom)

- **`src/collapsible_toolbar.rs`**: Alternative toolbar implementation with different UI patterns
  - `CollapsibleToolbar<Tab>`: A simpler toolbar-style collapsible component
  - `TabViewer` trait: Defines interface for custom tab content

- **`src/lib.rs`**: Library entry point that re-exports main types

### Key Features

- **Multi-directional panels**: Supports Left, Right, Top, Bottom positioning
- **Smooth animations**: Uses egui's animation system for expand/collapse transitions  
- **State persistence**: Panel states and sizes are saved/restored using egui's memory system
- **Width preservation**: Panel widths are maintained across collapse/expand cycles
- **Phosphor icons**: Uses `egui-phosphor` for professional-looking icons
- **Integration with egui_dock**: Full dock functionality with tabs and content management

### Dependencies

- `egui = "0.32.0"`: Core immediate mode GUI framework
- `egui_dock = "0.17.0"`: Dock panel system for egui
- `egui-phosphor = "0.10.0"`: Phosphor icon set
- `serde = "1.0"`: Serialization for state persistence
- `eframe = "0.32.0"` and `re_ui = "0.24.0"`: Used in examples only

### Example Usage Pattern

The library is designed to work with egui's immediate mode pattern:

```rust
let left_panel = CollapsibleDockPanel::new(PanelSide::Left, egui::Id::new("left_panel"))
    .with_dock_state(dock_state)
    .with_min_size(200.0)
    .add_button(CollapsibleButton::new("Files").with_icon("📁"));

left_panel.show(ctx, &mut tab_viewer);
```

### State Management

The library uses egui's built-in memory system for persistence, with both temporary and persistent storage options. State includes:
- Panel collapsed/expanded status
- Panel sizes and constraints
- Animation states

### Chinese Language Support

The codebase includes Chinese comments and the demo application has Chinese font setup, indicating this is primarily developed for Chinese-speaking users.