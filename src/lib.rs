//! # egui_collapsible_dock
//!
//! A collapsible dock panel system for egui with smooth animations and state persistence.
//!
//! This library provides collapsible dock panels that can be used with egui_dock to create
//! professional IDE-like interfaces with panels that can be collapsed to save space and
//! expanded when needed.
//!
//! ## Features
//!
//! - **Smooth animations**: Panels animate smoothly when collapsing/expanding
//! - **Width persistence**: Panel widths are preserved across collapse/expand cycles
//! - **State persistence**: Panel states are saved and restored across app restarts
//! - **Phosphor icons**: Beautiful icons from egui-phosphor for a professional look
//! - **re_ui integration**: Works seamlessly with the rerun design system
//! - **Flexible layout**: Supports left, right, top, and bottom panels
//!
//! ## Example
//!
//! ```rust,no_run
//! use egui_collapsible_dock::{CollapsibleDockPanel, CollapsibleButton, PanelSide};
//! use egui_dock::DockState;
//!
//! // Create a collapsible left panel
//! let left_panel = CollapsibleDockPanel::new(
//!     PanelSide::Left,
//!     egui::Id::new("left_panel"),
//! )
//! .with_dock_state(dock_state)
//! .with_min_size(200.0)
//! .add_button(
//!     CollapsibleButton::new("Files")
//!         .with_icon("📁")
//!         .with_tooltip("Browse files"),
//! );
//!
//! // Show the panel in your egui update loop
//! left_panel.show(ctx, &mut tab_viewer);
//! ```

pub mod dock_collapsible;

// Re-export main types for convenience
pub use dock_collapsible::{
    CollapsibleButton, CollapsibleDockPanel, CollapsibleDockState, PanelSide, PanelState,
};