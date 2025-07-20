use egui::Ui;
use serde::{Deserialize, Serialize};
use eframe::egui;
use egui_collapsible_dock::{CollapsibleDockPanel, CollapsibleButton, PanelSide};
use egui_dock::{DockArea, DockState, Style, TabViewer};

/// 应用设置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    enable_animations: bool,
    dark_theme: bool,
    show_tooltips: bool,
    show_line_numbers: bool,
    word_wrap: bool,
    font_size: i32,
    debug_mode: bool,
    auto_save: bool,
    auto_save_interval: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            enable_animations: true,
            dark_theme: false,
            show_tooltips: true,
            show_line_numbers: true,
            word_wrap: false,
            font_size: 14,
            debug_mode: false,
            auto_save: true,
            auto_save_interval: 30,
        }
    }
}

/// 面板标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    Left,
    Right,
    Bottom,
    Main,
}

/// 标签页内容类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TabContent {
    Files,
    Search,
    Diagnostics,
    History,
    Settings,
}

/// 示例标签页类型（包含面板信息以避免ID冲突）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DemoTab {
    pub panel_id: PanelId,
    pub content: TabContent,
    pub unique_id: String,
}

impl DemoTab {
    pub fn new(panel_id: PanelId, content: TabContent) -> Self {
        let unique_id = format!("{:?}_{:?}", panel_id, content);
        Self {
            panel_id,
            content,
            unique_id,
        }
    }
}

/// 显示标签页内容的函数
pub fn show_tab_content(ui: &mut Ui, tab: &DemoTab) {
    match &tab.content {
        TabContent::Files => {
            ui.heading("文件浏览器");
            ui.separator();

            ui.label("文件浏览器内容显示在这里");
            let src_id = egui::Id::new(&tab.unique_id).with("src_folder");
            egui::CollapsingHeader::new("📁 src")
                .id_salt(src_id)
                .show(ui, |ui| {
                    ui.label("📄 main.rs");
                    ui.label("📄 collapsible_toolbar.rs");
                    ui.label("📄 demo_tabs.rs");
                });

            let assets_id = egui::Id::new(&tab.unique_id).with("assets_folder");
            egui::CollapsingHeader::new("📁 assets")
                .id_salt(assets_id)
                .show(ui, |ui| {
                    ui.label("🖼️ icon.png");
                    ui.label("📄 config.toml");
                });

            let cargo_id = egui::Id::new(&tab.unique_id).with("cargo_folder");
            egui::CollapsingHeader::new("📁 .cargo")
                .id_salt(cargo_id)
                .show(ui, |ui| {
                    ui.label("📄 config.toml");
                });

            if ui.button("刷新文件列表").clicked() {
                // Refresh logic would go here
            }
        }
        TabContent::Search => {
            ui.heading("搜索");
            ui.separator();

            // Use per-tab state stored in egui memory
            let search_text_id = egui::Id::new(&tab.unique_id).with("search_text");
            let search_results_id = egui::Id::new(&tab.unique_id).with("search_results");

            let mut search_text = ui.data_mut(|d| d.get_persisted_mut_or_default::<String>(search_text_id).clone());
            let mut search_results = ui.data_mut(|d| d.get_persisted_mut_or_default::<Vec<String>>(search_results_id).clone());

            ui.horizontal(|ui| {
                ui.label("搜索:");
                ui.push_id(&tab.unique_id, |ui| {
                    let response = ui.text_edit_singleline(&mut search_text);
                    let search_clicked = ui.button("🔍").clicked();

                    if search_clicked || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        // Simulate search
                        search_results.clear();
                        if !search_text.is_empty() {
                            search_results.push(format!("📄 main.rs:15 - 找到 '{}'", search_text));
                            search_results.push(format!("📄 collapsible_toolbar.rs:42 - 找到 '{}'", search_text));
                            search_results.push(format!("📄 demo_tabs.rs:8 - 找到 '{}'", search_text));
                            search_results.push(format!("📄 README.md:25 - 找到 '{}'", search_text));
                        }
                        // Save updated search results
                        ui.data_mut(|d| d.insert_persisted(search_results_id, search_results.clone()));
                    }
                    // Save updated search text
                    ui.data_mut(|d| d.insert_persisted(search_text_id, search_text.clone()));
                });
            });

            ui.separator();

            if search_results.is_empty() {
                ui.label("输入搜索词并按回车键或点击搜索按钮");
            } else {
                ui.label(format!("找到 {} 个结果:", search_results.len()));
                let scroll_area_id = egui::Id::new(&tab.unique_id).with("search_results_scroll");
                egui::ScrollArea::vertical()
                    .id_salt(scroll_area_id)
                    .show(ui, |ui| {
                        for (i, result) in search_results.iter().enumerate() {
                            ui.push_id(i, |ui| {
                                if ui.selectable_label(false, result).clicked() {
                                    // Could open file here
                                }
                            });
                        }
                    });
            }
        }
        TabContent::Diagnostics => {
            ui.heading("诊断信息");
            ui.separator();

            ui.label("错误和警告:");

            // Simulate diagnostic information
            let diag_scroll_id = egui::Id::new(&tab.unique_id).with("diagnostics_scroll");
            egui::ScrollArea::vertical()
                .id_salt(diag_scroll_id)
                .show(ui, |ui| {
                    ui.push_id(&tab.unique_id, |ui| {
                        ui.group(|ui| {
                            ui.colored_label(egui::Color32::RED, "❌ 错误: 找不到函数 'foo'");
                            ui.colored_label(egui::Color32::RED, "❌ 错误: 第42行类型不匹配");
                            ui.colored_label(egui::Color32::YELLOW, "⚠️ 警告: 未使用的变量 'x'");
                            ui.colored_label(egui::Color32::YELLOW, "⚠️ 警告: 检测到死代码");
                            ui.colored_label(egui::Color32::YELLOW, "⚠️ 警告: 使用了已弃用的方法");
                            ui.colored_label(egui::Color32::BLUE, "ℹ️ 信息: 编译完成");
                            ui.colored_label(egui::Color32::GREEN, "✅ 成功: 所有测试通过");
                        });
                    });
                });

            ui.separator();
            ui.push_id((&tab.unique_id, "diagnostics_buttons"), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("清除诊断").clicked() {
                        // Clear logic would go here
                    }
                    if ui.button("刷新").clicked() {
                        // Refresh diagnostics
                    }
                });
            });
        }
        TabContent::History => {
            ui.heading("操作历史");
            ui.separator();

            ui.label("最近的操作:");

            // Simulate history records
            let history_scroll_id = egui::Id::new(&tab.unique_id).with("history_scroll");
            egui::ScrollArea::vertical()
                .id_salt(history_scroll_id)
                .show(ui, |ui| {
                    let operations = [
                        "打开文件: main.rs",
                        "执行搜索: 'CollapsibleToolbar'",
                        "切换到诊断标签页",
                        "更新设置",
                        "保存文件: demo_tabs.rs",
                        "构建成功完成",
                        "启动测试运行",
                        "Git 提交: '添加可折叠工具栏'",
                        "更新依赖: egui 0.32.0",
                        "创建项目",
                    ];

                    for (i, operation) in operations.iter().enumerate() {
                        ui.push_id(i, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", i + 1));
                                ui.label(format!("{} - 2024-07-20 {:02}:{:02}", operation, 14 + i % 8, i * 3 % 60));
                            });
                        });
                    }
                });

            ui.separator();
            ui.push_id((&tab.unique_id, "history_buttons"), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("清除历史").clicked() {
                        // Clear history logic would go here
                    }
                    if ui.button("导出历史").clicked() {
                        // Export history logic would go here
                    }
                });
            });
        }
        TabContent::Settings => {
            ui.heading("设置");
            ui.separator();

            // Use per-tab settings state stored in egui memory
            let settings_id = egui::Id::new(&tab.unique_id).with("settings_state");
            let mut settings = ui.data_mut(|d| d.get_persisted_mut_or_default::<AppSettings>(settings_id).clone());

            ui.push_id((&tab.unique_id, "interface_group"), |ui| {
                ui.group(|ui| {
                    ui.label("界面设置");
                    ui.checkbox(&mut settings.enable_animations, "启用动画");
                    ui.checkbox(&mut settings.dark_theme, "深色主题");
                    ui.checkbox(&mut settings.show_tooltips, "显示工具提示");
                });
            });

            ui.push_id((&tab.unique_id, "editor_group"), |ui| {
                ui.group(|ui| {
                    ui.label("编辑器设置");
                    ui.checkbox(&mut settings.show_line_numbers, "显示行号");
                    ui.checkbox(&mut settings.word_wrap, "自动换行");
                    ui.horizontal(|ui| {
                        ui.label("字体大小:");
                        ui.add(egui::Slider::new(&mut settings.font_size, 8..=24));
                    });
                });
            });

            ui.push_id((&tab.unique_id, "advanced_group"), |ui| {
                ui.group(|ui| {
                    ui.label("高级设置");
                    ui.checkbox(&mut settings.debug_mode, "调试模式");
                    ui.checkbox(&mut settings.auto_save, "自动保存");
                    ui.horizontal(|ui| {
                        ui.label("自动保存间隔 (秒):");
                        ui.add(egui::Slider::new(&mut settings.auto_save_interval, 5..=300));
                    });
                });
            });

            ui.separator();
            ui.push_id((&tab.unique_id, "settings_buttons"), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("重置为默认").clicked() {
                        settings = AppSettings::default();
                        ui.data_mut(|d| d.insert_persisted(settings_id, settings.clone()));
                    }
                    if ui.button("应用设置").clicked() {
                        // Settings application logic would go here
                        ui.ctx().request_repaint(); // Trigger repaint
                    }
                });
            });

            // Show current settings status
            ui.separator();
            let status_header_id = egui::Id::new(&tab.unique_id).with("status_header");
            egui::CollapsingHeader::new("当前设置状态")
                .id_salt(status_header_id)
                .show(ui, |ui| {
                    ui.label(format!("动画: {}", if settings.enable_animations { "已启用" } else { "已禁用" }));
                    ui.label(format!("主题: {}", if settings.dark_theme { "深色" } else { "浅色" }));
                    ui.label(format!("字体大小: {}", settings.font_size));
                    ui.label(format!("自动保存: {} ({}秒)",
                        if settings.auto_save { "已启用" } else { "已禁用" },
                        settings.auto_save_interval));
                });

            // Save updated settings
            ui.data_mut(|d| d.insert_persisted(settings_id, settings));
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("标签页演示 - Egui 可折叠停靠面板"),
        ..Default::default()
    };

    eframe::run_native(
        "标签页演示 - Egui 可折叠停靠面板",
        options,
        Box::new(|cc| {
            // 设置中文字体支持
            // setup_chinese_fonts_robust(&cc.egui_ctx);
            Ok(Box::new(DemoTabsApp::default()))
        }),
    )
}

/// 设置中文字体支持 - 在 re_ui 样式基础上添加中文字体
pub fn setup_chinese_fonts_robust(ctx: &egui::Context) {
    let mut font_definitions = egui::FontDefinitions::default();
    
    // 添加中文字体 - 使用系统字体或嵌入字体
    #[cfg(target_os = "windows")]
    {
        // Windows 系统字体
        if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/msyh.ttc") {
            font_definitions.font_data.insert(
                "Microsoft YaHei".into(),
                egui::FontData::from_owned(font_data).into(),
            );
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS 系统字体
        if let Ok(font_data) = std::fs::read("fonts/AlibabaPuHuiTi-2-65-Medium.ttf") {
            font_definitions.font_data.insert(
                "PingFang SC".into(),
                egui::FontData::from_owned(font_data).into(),
            );
        }
    }
    
    // 嵌入的备用中文字体
    // font_definitions.font_data.insert(
    //     "NotoSansCJK".into(),
    //     egui::FontData::from_static(include_bytes!("../fonts/NotoSansCJK-Regular.ttf")),
    // );
    
    // 设置字体优先级
    let font_list = vec![
        #[cfg(target_os = "windows")]
        "Microsoft YaHei".to_owned(),
        #[cfg(target_os = "macos")]
        "PingFang SC".to_owned(),
    ];
    
    font_definitions.families.insert(
        egui::FontFamily::Proportional,
        font_list.clone(),
    );
    font_definitions.families.insert(
        egui::FontFamily::Monospace,
        font_list,
    );

    egui_phosphor::add_to_fonts(&mut font_definitions, egui_phosphor::Variant::Regular);
    
    ctx.set_fonts(font_definitions);
    
    // 强制重新布局
    ctx.request_repaint();
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
        show_tab_content(ui, tab);
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        true // 允许关闭标签页
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(format!("tab_{}", &tab.unique_id))
    }
}

struct DemoTabsApp {
    dock_state: DockState<DemoTab>,
    left_panel: CollapsibleDockPanel<DemoTabViewer>,
    right_panel: CollapsibleDockPanel<DemoTabViewer>,
    bottom_panel: CollapsibleDockPanel<DemoTabViewer>,
    style_initialized: bool,
}

impl Default for DemoTabsApp {
    fn default() -> Self {
        // 创建主 dock 状态，包含所有类型的标签页
        let mut dock_state = DockState::new(vec![DemoTab::new(PanelId::Main, TabContent::Files)]);
        dock_state.main_surface_mut().push_to_focused_leaf(DemoTab::new(PanelId::Main, TabContent::Search));
        dock_state.main_surface_mut().push_to_focused_leaf(DemoTab::new(PanelId::Main, TabContent::Diagnostics));

        // 创建左侧面板
        let mut left_dock = DockState::new(vec![DemoTab::new(PanelId::Left, TabContent::Files)]);
        left_dock.main_surface_mut().push_to_focused_leaf(DemoTab::new(PanelId::Left, TabContent::Search));
        let left_panel = CollapsibleDockPanel::new(
            PanelSide::Left,
            egui::Id::new("collapsible_left_panel"),
        )
        .with_dock_state(left_dock)
        .with_min_size(250.0)
        .add_button(
            CollapsibleButton::new("文件")
                .with_icon("📁")
                .with_tooltip("浏览文件"),
        )
        .add_button(
            CollapsibleButton::new("搜索")
                .with_icon("🔍")
                .with_tooltip("搜索文件和内容"),
        );

        // 创建右侧面板
        let mut right_dock = DockState::new(vec![DemoTab::new(PanelId::Right, TabContent::Diagnostics)]);
        right_dock.main_surface_mut().push_to_focused_leaf(DemoTab::new(PanelId::Right, TabContent::History));
        let right_panel = CollapsibleDockPanel::new(
            PanelSide::Right,
            egui::Id::new("collapsible_right_panel"),
        )
        .with_dock_state(right_dock)
        .with_min_size(280.0)
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
        .with_min_size(200.0)
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

impl eframe::App for DemoTabsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 只在第一次初始化时设置样式和字体，避免字体闪烁
        if !self.style_initialized {
            // 应用 re_ui 设计系统
            re_ui::apply_style_and_install_loaders(ctx);

            // 设置中文字体（只设置一次）
            setup_chinese_fonts_robust(ctx);

            self.style_initialized = true;
        }

        // 顶部菜单栏
        egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("📋 视图", |ui| {
                        let left_text = if self.left_panel.is_collapsed() { "▶ 展开左侧面板" } else { "◀ 折叠左侧面板" };
                        if ui.button(left_text).clicked() {
                            self.left_panel.toggle();
                            ui.close();
                        }

                        let right_text = if self.right_panel.is_collapsed() { "◀ 展开右侧面板" } else { "▶ 折叠右侧面板" };
                        if ui.button(right_text).clicked() {
                            self.right_panel.toggle();
                            ui.close();
                        }

                        let bottom_text = if self.bottom_panel.is_collapsed() { "▲ 展开底部面板" } else { "▼ 折叠底部面板" };
                        if ui.button(bottom_text).clicked() {
                            self.bottom_panel.toggle();
                            ui.close();
                        }

                        ui.separator();
                        if ui.button("📤 全部折叠").clicked() {
                            self.left_panel.set_collapsed(true);
                            self.right_panel.set_collapsed(true);
                            self.bottom_panel.set_collapsed(true);
                            ui.close();
                        }
                        if ui.button("📥 全部展开").clicked() {
                            self.left_panel.set_collapsed(false);
                            self.right_panel.set_collapsed(false);
                            self.bottom_panel.set_collapsed(false);
                            ui.close();
                        }
                    });

                    ui.separator();
                    ui.strong("🏷️ 标签页演示 - 可折叠停靠面板");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(egui::Color32::from_rgb(120, 200, 120), format!("帧率: {:.1}", ctx.input(|i| 1.0 / i.stable_dt)));
                        ui.separator();
                        ui.label(format!("面板: 左:{} 右:{} 下:{}",
                            if self.left_panel.is_collapsed() { "❌" } else { "✅" },
                            if self.right_panel.is_collapsed() { "❌" } else { "✅" },
                            if self.bottom_panel.is_collapsed() { "❌" } else { "✅" }
                        ));
                    });
                });
            });

        // 键盘快捷键处理
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

        // 显示可折叠面板
        self.left_panel.show(ctx, &mut DemoTabViewer);
        self.right_panel.show(ctx, &mut DemoTabViewer);
        self.bottom_panel.show(ctx, &mut DemoTabViewer);

        // 中央面板
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.heading("🏷️ Demo Tabs - 标签页内容演示");
                ui.separator();

                ui.label("这个演示展示了各种标签页内容类型：");
                ui.label("• 📁 文件 - 文件浏览器");
                ui.label("• 🔍 搜索 - 搜索功能");
                ui.label("• ⚠️ 诊断 - 诊断信息");
                ui.label("• 📜 历史 - 操作历史");
                ui.label("• ⚙️ 设置 - 应用设置");

                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.strong("键盘快捷键:");
                    ui.label("• F1: 切换左侧面板");
                    ui.label("• F2: 切换右侧面板");
                    ui.label("• F3: 切换底部面板");
                });

                ui.add_space(20.0);

                // 显示主 dock 区域
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


