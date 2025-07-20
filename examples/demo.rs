mod demo_tabs;

use demo_tabs::{DemoTab, PanelId, TabContent};
use egui_collapsible_dock::{CollapsibleDockPanel, CollapsibleButton, PanelSide};
use eframe::egui;
use egui_dock::{DockArea, DockState, Style, TabViewer};


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("可折叠停靠面板演示"),
        ..Default::default()
    };

    eframe::run_native(
        "可折叠停靠面板演示",
        options,
        Box::new(|cc| {
            // 设置中文字体支持
            setup_chinese_fonts(&cc.egui_ctx);
            Ok(Box::new(DemoApp::default()))
        }),
    )
}

/// 设置中文字体支持 - 在 re_ui 样式基础上添加中文字体
fn setup_chinese_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions, FontFamily};
    use std::sync::Once;

    static FONT_SETUP: Once = Once::new();
    static mut CHINESE_FONT_DATA: Option<Vec<u8>> = None;

    // 只在第一次调用时加载字体数据
    FONT_SETUP.call_once(|| {
        let chinese_font_paths = [
            // macOS 系统字体
            "/System/Library/Fonts/PingFang.ttc",        // 苹方
            "/System/Library/Fonts/STHeiti Light.ttc",   // 华文黑体
            "/System/Library/Fonts/STSong.ttc",          // 华文宋体
            "/System/Library/Fonts/Hiragino Sans GB.ttc", // 冬青黑体
            // Windows 系统字体
            "C:/Windows/Fonts/msyh.ttc",                 // 微软雅黑
            "C:/Windows/Fonts/simsun.ttc",               // 宋体
            // Linux 系统字体
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", // 文泉驿微米黑
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", // DejaVu Sans
        ];

        for font_path in &chinese_font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                unsafe {
                    CHINESE_FONT_DATA = Some(font_data);
                }
                println!("成功加载中文字体: {}", font_path);
                return;
            }
        }

        println!("未找到系统中文字体，使用默认字体（egui 默认字体已支持基本中文显示）");
    });

    // 清空之前的字体定义，重新开始配置
    let mut fonts = FontDefinitions::default();

    // 清空默认字体族配置
    fonts.families.clear();

    unsafe {
        if let Some(ref font_data) = CHINESE_FONT_DATA {
            // 添加中文字体数据
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                FontData::from_owned(font_data.clone()).into(),
            );

            // 重新设置字体族，优先使用中文字体
            fonts.families.insert(FontFamily::Proportional, vec!["chinese_font".to_owned()]);
            fonts.families.insert(FontFamily::Monospace, vec!["chinese_font".to_owned()]);

            // 重新设置字体配置
            ctx.set_fonts(fonts);
        } else {
            // 如果没有中文字体，使用默认配置
            ctx.set_fonts(fonts);
        }
    }
}

struct DemoTabViewer;

impl TabViewer for DemoTabViewer {
    type Tab = DemoTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match &tab.content {
            TabContent::Files => "📁 文件".into(),
            TabContent::Search => "🔍 搜索".into(),
            TabContent::Diagnostics => "⚠️ 诊断".into(),
            TabContent::History => "📜 历史".into(),
            TabContent::Settings => "⚙️ 设置".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        demo_tabs::show_tab_content(ui, tab);
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false // Tabs are not closeable in this demo
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        // 使用标签页的唯一 ID，确保在所有面板中都是唯一的
        egui::Id::new(format!("tab_{}", &tab.unique_id))
    }
}

struct DemoApp {
    dock_state: DockState<DemoTab>,
    left_panel: CollapsibleDockPanel<DemoTabViewer>,
    right_panel: CollapsibleDockPanel<DemoTabViewer>,
    bottom_panel: CollapsibleDockPanel<DemoTabViewer>,
    style_initialized: bool,
}

impl Default for DemoApp {
    fn default() -> Self {
        // 创建主 dock 状态
        let dock_state = DockState::new(vec![DemoTab::new(PanelId::Main, TabContent::Files)]);

        // 创建左侧面板
        let left_dock = DockState::new(vec![DemoTab::new(PanelId::Left, TabContent::Search)]);
        let left_panel = CollapsibleDockPanel::new(
            PanelSide::Left,
            egui::Id::new("collapsible_left_panel"),
        )
        .with_dock_state(left_dock)
        .with_min_size(200.0)
        .add_button(
            CollapsibleButton::new("搜索")
                .with_icon("🔍")
                .with_tooltip("搜索文件和内容"),
        )
        .add_button(
            CollapsibleButton::new("文件")
                .with_icon("📁")
                .with_tooltip("浏览文件"),
        );

        // 创建右侧面板
        let mut right_dock = DockState::new(vec![DemoTab::new(PanelId::Right, TabContent::Diagnostics)]);
        right_dock.main_surface_mut().push_to_focused_leaf(DemoTab::new(PanelId::Right, TabContent::History));
        let right_panel = CollapsibleDockPanel::new(
            PanelSide::Right,
            egui::Id::new("collapsible_right_panel"),
        )
        .with_dock_state(right_dock)
        .with_min_size(250.0)
        .add_button(
            CollapsibleButton::new("诊断")
                .with_icon("⚠️")
                .with_tooltip("查看诊断和错误"),
        )
        .add_button(
            CollapsibleButton::new("历史")
                .with_icon("📜")
                .with_tooltip("查看命令历史"),
        );

        // 创建底部面板
        let bottom_dock = DockState::new(vec![DemoTab::new(PanelId::Bottom, TabContent::Settings)]);
        let bottom_panel = CollapsibleDockPanel::new(
            PanelSide::Bottom,
            egui::Id::new("collapsible_bottom_panel"),
        )
        .with_dock_state(bottom_dock)
        .with_min_size(150.0)
        .add_button(
            CollapsibleButton::new("设置")
                .with_icon("⚙️")
                .with_tooltip("应用程序设置"),
        );

        Self {
            dock_state,
            left_panel,
            right_panel,
            bottom_panel,
            style_initialized: false,
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 只在第一次初始化时设置样式和字体，避免字体闪烁
        if !self.style_initialized {
            // 应用 re_ui 设计系统
            re_ui::apply_style_and_install_loaders(ctx);

            // 设置中文字体（只设置一次）
            setup_chinese_fonts(ctx);

            self.style_initialized = true;
        }
        

        // Top menu bar with re_ui styling applied automatically
        egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("📋 View", |ui| {
                    let left_text = if self.left_panel.is_collapsed() { "▶ Expand Left Panel" } else { "◀ Collapse Left Panel" };
                    if ui.button(left_text).clicked() {
                        self.left_panel.toggle();
                        ui.close();
                    }

                    let right_text = if self.right_panel.is_collapsed() { "◀ Expand Right Panel" } else { "▶ Collapse Right Panel" };
                    if ui.button(right_text).clicked() {
                        self.right_panel.toggle();
                        ui.close();
                    }

                    let bottom_text = if self.bottom_panel.is_collapsed() { "▲ Expand Bottom Panel" } else { "▼ Collapse Bottom Panel" };
                    if ui.button(bottom_text).clicked() {
                        self.bottom_panel.toggle();
                        ui.close();
                    }

                    ui.separator();
                    if ui.button("📤 Collapse All").clicked() {
                        self.left_panel.set_collapsed(true);
                        self.right_panel.set_collapsed(true);
                        self.bottom_panel.set_collapsed(true);
                        ui.close();
                    }
                    if ui.button("📥 Expand All").clicked() {
                        self.left_panel.set_collapsed(false);
                        self.right_panel.set_collapsed(false);
                        self.bottom_panel.set_collapsed(false);
                        ui.close();
                    }
                });

                ui.separator();
                ui.strong("🔧 Egui Collapsible Dock Demo");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(egui::Color32::from_rgb(120, 200, 120), format!("FPS: {:.1}", ctx.input(|i| 1.0 / i.stable_dt)));
                    ui.separator();
                    ui.label(format!("Panels: L:{} R:{} B:{}",
                        if self.left_panel.is_collapsed() { "❌" } else { "✅" },
                        if self.right_panel.is_collapsed() { "❌" } else { "✅" },
                        if self.bottom_panel.is_collapsed() { "❌" } else { "✅" }
                    ));
                });
            });
        });

        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F1) {
                self.left_panel.toggle();
            }
            if i.key_pressed(egui::Key::F2) {
                self.right_panel.toggle();
            }
            if i.key_pressed(egui::Key::F3) {
                self.bottom_panel.toggle();
            }
        });

        // Show collapsible panels with separate TabViewer instances
        self.left_panel.show(ctx, &mut DemoTabViewer);
        self.right_panel.show(ctx, &mut DemoTabViewer);
        self.bottom_panel.show(ctx, &mut DemoTabViewer);

        // Central panel with re_ui styling applied automatically
        egui::CentralPanel::default()
            .show(ctx, |ui| {
            ui.heading("🔧 Egui Collapsible Dock Demo");
            ui.separator();

            ui.label("This demo showcases collapsible dock panels using egui_dock with re_ui theming.");
            ui.label("Use the View menu to toggle panels, or try these keyboard shortcuts:");
            ui.label("• F1: Toggle Left Panel");
            ui.label("• F2: Toggle Right Panel");
            ui.label("• F3: Toggle Bottom Panel");

            ui.add_space(20.0);

            ui.group(|ui| {
                ui.strong("Panel Status:");
                ui.label(format!("Left Panel: {}", if self.left_panel.is_collapsed() { "Collapsed ❌" } else { "Expanded ✅" }));
                ui.label(format!("Right Panel: {}", if self.right_panel.is_collapsed() { "Collapsed ❌" } else { "Expanded ✅" }));
                ui.label(format!("Bottom Panel: {}", if self.bottom_panel.is_collapsed() { "Collapsed ❌" } else { "Expanded ✅" }));
            });

            ui.add_space(20.0);

            ui.group(|ui| {
                ui.strong("Features:");
                ui.label("✅ Smooth expand/collapse animations");
                ui.label("✅ State persistence across app restarts");
                ui.label("✅ Real egui_dock integration");
                ui.label("✅ Keyboard shortcuts support");
                ui.label("✅ Unique ID management (no conflicts)");
                ui.label("✅ Responsive layout");
                ui.label("✅ Professional re_ui design system theming");
                ui.label("✅ Phosphor icons for professional appearance");
            });

            ui.add_space(20.0);

            // Show the main dock area with unique ID
            ui.push_id("main_dock_area", |ui| {
                DockArea::new(&mut self.dock_state)
                    .id(egui::Id::new("main_dock_area_unique"))
                    .style(Style::from_egui(ctx.style().as_ref()))
                    .show_leaf_collapse_buttons(false)  // 直接禁用 collapse 按钮
                    .show_inside(ui, &mut DemoTabViewer);
            });
        });
    }
}