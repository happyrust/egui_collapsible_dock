# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-20

### Added
- Initial release of egui_collapsible_dock
- Collapsible dock panel system for egui with smooth animations
- State persistence functionality
- Support for collapsible toolbar components
- Demo examples showcasing the functionality
- MIT OR Apache-2.0 dual licensing

### Features
- **CollapsibleDock**: Main dock component with collapsible panels
- **CollapsibleToolbar**: Toolbar component that can be collapsed/expanded
- **State Management**: Automatic state persistence across sessions
- **Smooth Animations**: Fluid collapse/expand animations
- **egui Integration**: Seamless integration with egui ecosystem

### Dependencies
- egui 0.32.0
- egui_dock 0.17.0
- egui-phosphor 0.10.0
- serde 1.0 (with derive features)

### Examples
- `demo.rs`: Basic usage demonstration
- `demo_tabs.rs`: Advanced tabbed interface example

[Unreleased]: https://github.com/happyrust/egui_collapsible_dock/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/happyrust/egui_collapsible_dock/releases/tag/v0.1.0
