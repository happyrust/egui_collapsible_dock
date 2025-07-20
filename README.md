# egui_collapsible_dock

A collapsible dock panel system for egui with smooth animations and state persistence.

## Features

- ✅ **Smooth Animations**: Fluid expand/collapse effects using egui's animation system
- ✅ **Width Persistence**: Panel widths are preserved across collapse/expand cycles
- ✅ **State Persistence**: Panel states are saved and restored across app restarts
- ✅ **Phosphor Icons**: Beautiful icons from egui-phosphor for a professional look
- ✅ **re_ui Integration**: Works seamlessly with the rerun design system
- ✅ **Multi-directional Support**: Supports panels on left, right, top, and bottom sides
- ✅ **Resizable Panels**: Drag to resize panels when expanded
- ✅ **egui_dock Integration**: Full dock functionality with tabs and content management

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
egui_collapsible_dock = "0.1.0"
egui = "0.32.0"
egui_dock = "0.17.0"
egui-phosphor = "0.10.0"
```

## Library Structure

```
src/
├── lib.rs                     # Library API exports
└── dock_collapsible.rs        # Collapsible dock panel implementation

examples/
├── demo.rs                    # Demo application
└── demo_tabs.rs              # Demo tab content
```

## Core Components

### CollapsibleDockPanel

The main panel component that integrates with egui_dock to provide collapsible functionality:

- Support for four panel directions (left, right, top, bottom)
- State persistence configuration
- Custom frame styling
- Configurable minimum size
- Resizable panels with width persistence
- Smooth animations with egui's animation system

### CollapsibleButton

Buttons that appear when panels are collapsed, allowing quick access to panel content:

- Phosphor icon integration
- Tooltip support
- Click handling for panel expansion

## Quick Start

### Basic Usage

```rust
use egui_collapsible_dock::{CollapsibleDockPanel, CollapsibleButton, PanelSide};
use egui_dock::DockState;

// Create a collapsible left panel
let left_panel = CollapsibleDockPanel::new(
    PanelSide::Left,
    egui::Id::new("left_panel"),
)
.with_dock_state(dock_state)
.with_min_size(200.0)
.add_button(
    CollapsibleButton::new("Files")
        .with_icon("📁")
        .with_tooltip("Browse files"),
);

// Show the panel in your egui update loop
left_panel.show(ctx, &mut tab_viewer);
```

### 自定义标签页

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum MyTab {
    Files,
    Search,
    Settings,
}

struct MyTabViewer;

impl TabViewer for MyTabViewer {
    type Tab = MyTab;
    
    fn title(&self, tab: &Self::Tab) -> String {
        match tab {
            MyTab::Files => "📁 文件".to_string(),
            MyTab::Search => "🔍 搜索".to_string(),
            MyTab::Settings => "⚙️ 设置".to_string(),
        }
    }
    
    fn ui(&mut self, ui: &mut Ui, tab: &Self::Tab) {
        match tab {
            MyTab::Files => {
                ui.heading("文件浏览器");
                // 文件浏览器内容
            }
            // ... 其他标签页内容
        }
    }
}
```

## Running the Demo

To see the library in action, run the demo example:

```bash
# Clone the project
git clone <repository-url>
cd collapsible_toolbar_demo

# Run the demo with required features
cargo run --example demo --features examples
```

## 依赖项

- `eframe = "0.32.0"` - egui 应用框架
- `egui = "0.32.0"` - 即时模式 GUI 库
- `egui_dock = "0.17.0"` - egui 停靠面板扩展
- `serde = { version = "1.0", features = ["derive"] }` - 序列化支持

## 交互说明

1. **展开工具栏**: 点击收叠状态下的标签页按钮
2. **切换标签页**: 在展开状态下点击不同的标签页
3. **收叠工具栏**: 点击展开状态下的 ✕ 按钮，或点击当前选中的标签页
4. **调整大小**: 在展开状态下拖拽面板边缘
5. **查看提示**: 在收叠状态下悬停在按钮上查看完整标题

## 技术实现

### 动画系统

使用 egui 的 `show_animated` 方法实现平滑的展开/收叠动画：

```rust
panel.show_animated(ctx, is_expanded, |ui| {
    // 内容渲染
})
```

### 状态管理

使用 egui 的内存系统进行状态持久化：

```rust
// 保存状态
ctx.memory_mut(|mem| {
    if self.persist {
        mem.data.insert_persisted(state_id, state);
    } else {
        mem.data.insert_temp(state_id, state);
    }
});
```

### 响应式布局

根据面板方向自动调整布局和尺寸约束：

```rust
match self.side {
    PanelSide::Left | PanelSide::Right => {
        // 使用 SidePanel 和宽度约束
    }
    PanelSide::Top | PanelSide::Bottom => {
        // 使用 TopBottomPanel 和高度约束
    }
}
```

## 扩展建议

1. **拖拽重排**: 添加标签页拖拽重新排序功能
2. **右键菜单**: 为标签页添加上下文菜单
3. **标签页关闭**: 实现可关闭标签页的功能
4. **主题支持**: 添加多主题切换支持
5. **快捷键**: 添加键盘快捷键支持

## 许可证

MIT License
