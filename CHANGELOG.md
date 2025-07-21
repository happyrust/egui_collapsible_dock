# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 统一的面板渲染系统，减少代码重复
- 动态面板 ID 机制，解决展开/折叠状态冲突
- 改进的面板尺寸管理和持久化

### Fixed
- 修复面板展开时使用 `min_width` 而不是 `default_width` 的问题
- 修复底部面板无法向上调整大小的问题
- 修复 `get_panel_size` 方法的默认值不一致问题
- 解决 egui 内部状态冲突导致的宽度限制问题

### Changed
- 重构面板渲染逻辑，将四个方向的面板统一到 `show_panel_unified` 方法
- 更新默认面板尺寸从 50.0 到 250.0
- 更新最小面板尺寸从 50.0 到 150.0
- 改进面板 ID 命名策略，为不同状态使用不同 ID

### Technical Details
- 为展开和折叠状态使用不同的面板 ID (`{side}_expanded` / `{side}_collapsed`)
- 统一所有面板的最大尺寸处理逻辑 (`max_size.unwrap_or(f32::INFINITY)`)
- 改进面板尺寸保存机制，确保用户调整被正确持久化
- 优化代码结构，减少约 70 行重复代码

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
