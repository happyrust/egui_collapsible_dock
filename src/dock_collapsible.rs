use egui::{Context, Frame, Id, Response, Ui, Vec2};
use egui_dock::{DockState, TabViewer};
use egui_phosphor::regular as phosphor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 缓动函数：ease-in-out-cubic
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// 面板方向枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// 单个面板的折叠状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    /// 是否折叠
    pub collapsed: bool,
    /// 面板尺寸（展开时）
    pub size: f32,
    /// 最小尺寸
    pub min_size: f32,
    /// 最大尺寸
    pub max_size: Option<f32>,
    /// 是否可调整大小
    pub resizable: bool,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            collapsed: false,
            size: 300.0, // 增加默认宽度，确保有足够空间
            min_size: 150.0,
            max_size: None,
            resizable: true,
        }
    }
}

/// 可折叠 Dock 状态管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapsibleDockState {
    /// 各个面板的状态
    pub panels: HashMap<PanelSide, PanelState>,
    /// 动画持续时间（秒）
    pub animation_duration: f32,
    /// 是否启用状态持久化
    pub persist_state: bool,
}

impl Default for CollapsibleDockState {
    fn default() -> Self {
        let mut panels = HashMap::new();
        panels.insert(PanelSide::Left, PanelState::default());
        panels.insert(PanelSide::Right, PanelState::default());
        panels.insert(PanelSide::Top, PanelState::default());
        panels.insert(PanelSide::Bottom, PanelState::default());

        Self {
            panels,
            animation_duration: 0.2,
            persist_state: true,
        }
    }
}

impl CollapsibleDockState {
    /// 创建新的可折叠 Dock 状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置面板状态
    pub fn set_panel_collapsed(&mut self, side: PanelSide, collapsed: bool) {
        if let Some(panel) = self.panels.get_mut(&side) {
            panel.collapsed = collapsed;
        }
    }

    /// 获取面板是否折叠
    pub fn is_panel_collapsed(&self, side: PanelSide) -> bool {
        self.panels.get(&side).map(|p| p.collapsed).unwrap_or(false)
    }

    /// 切换面板折叠状态
    pub fn toggle_panel(&mut self, side: PanelSide) {
        if let Some(panel) = self.panels.get_mut(&side) {
            panel.collapsed = !panel.collapsed;
        }
    }

    /// 设置面板尺寸
    pub fn set_panel_size(&mut self, side: PanelSide, size: f32) {
        if let Some(panel) = self.panels.get_mut(&side) {
            // 确保尺寸在合理范围内，但不强制使用min_size作为最小值
            let validated_size = if size < 100.0 {
                // 如果尺寸太小，使用一个合理的默认值
                300.0
            } else {
                size
            };

            panel.size = validated_size;
            if let Some(max_size) = panel.max_size {
                panel.size = panel.size.min(max_size);
            }

            // 调试信息
            // println!("set_panel_size: side={:?}, old_size={}, new_size={}",
            //     side, panel.size, validated_size);
        } else {
            // println!("set_panel_size: panel not found for side={:?}", side);
        }
    }

    /// 获取面板尺寸
    pub fn get_panel_size(&self, side: PanelSide) -> f32 {
        self.panels
            .get(&side)
            .map(|p| p.size)
            .unwrap_or(PanelState::default().size)
    }

    /// 保存状态到 egui 内存
    pub fn save_to_memory(&self, ctx: &Context, id: Id) {
        if self.persist_state {
            ctx.memory_mut(|mem| {
                mem.data
                    .insert_persisted(id.with("dock_state"), self.clone());
            });
        }
    }

    /// 从 egui 内存加载状态
    pub fn load_from_memory(ctx: &Context, id: Id) -> Self {
        ctx.memory_mut(|mem| {
            mem.data
                .get_persisted_mut_or_default::<Self>(id.with("dock_state"))
                .clone()
        })
    }
}

/// 可折叠面板按钮配置
#[derive(Debug, Clone)]
pub struct CollapsibleButton {
    /// 按钮文本
    pub text: String,
    /// 按钮图标
    pub icon: Option<String>,
    /// 工具提示
    pub tooltip: Option<String>,
    /// 是否选中
    pub selected: bool,
}

impl CollapsibleButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            tooltip: None,
            selected: false,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// 可折叠 Dock 面板
pub struct CollapsibleDockPanel<Tab: TabViewer> {
    /// 面板方向
    side: PanelSide,
    /// Dock 状态
    dock_state: DockState<Tab::Tab>,
    /// 折叠状态
    collapsible_state: CollapsibleDockState,
    /// 状态 ID
    state_id: Id,
    /// 面板框架样式
    frame: Option<Frame>,
    /// 按钮列表（折叠时显示）
    buttons: Vec<CollapsibleButton>,
    /// 前一帧的折叠状态（用于检测状态变化）
    previous_collapsed: bool,
    /// 状态是否已加载
    state_loaded: bool,
    /// 当前活动的按钮索引
    active_button_index: Option<usize>,
}

impl<Tab: TabViewer> CollapsibleDockPanel<Tab> {
    /// 创建新的可折叠 Dock 面板
    pub fn new(side: PanelSide, state_id: Id) -> Self {
        Self {
            side,
            dock_state: DockState::new(vec![]),
            collapsible_state: CollapsibleDockState::new(),
            state_id,
            frame: None,
            buttons: Vec::new(),
            previous_collapsed: false,
            state_loaded: false,
            active_button_index: Some(0), // 默认第一个按钮为活动状态
        }
    }

    /// 设置 Dock 状态
    pub fn with_dock_state(mut self, dock_state: DockState<Tab::Tab>) -> Self {
        self.dock_state = dock_state;
        self
    }

    /// 设置面板框架
    pub fn with_frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }

    /// 添加折叠按钮
    pub fn add_button(mut self, button: CollapsibleButton) -> Self {
        self.buttons.push(button);
        self
    }

    /// 设置面板最小尺寸
    pub fn with_min_size(mut self, min_size: f32) -> Self {
        if let Some(panel) = self.collapsible_state.panels.get_mut(&self.side) {
            panel.min_size = min_size;
            // 如果当前尺寸小于最小尺寸，设置一个合理的默认展开宽度
            if panel.size < min_size * 1.5 {
                panel.size = (min_size * 2.0).max(300.0); // 确保有足够的展开宽度
            }
        }
        self
    }

    /// 设置面板最大尺寸
    pub fn with_max_size(mut self, max_size: f32) -> Self {
        if let Some(panel) = self.collapsible_state.panels.get_mut(&self.side) {
            panel.max_size = Some(max_size);
        }
        self
    }

    /// 设置是否可调整大小
    pub fn resizable(mut self, resizable: bool) -> Self {
        if let Some(panel) = self.collapsible_state.panels.get_mut(&self.side) {
            panel.resizable = resizable;
        }
        self
    }

    /// 获取当前折叠状态
    pub fn is_collapsed(&self) -> bool {
        self.collapsible_state.is_panel_collapsed(self.side)
    }

    /// 切换折叠状态
    pub fn toggle(&mut self) {
        self.collapsible_state.toggle_panel(self.side);
    }

    /// 设置折叠状态
    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsible_state
            .set_panel_collapsed(self.side, collapsed);
    }

    /// 获取面板尺寸
    pub fn get_size(&self) -> f32 {
        self.collapsible_state.get_panel_size(self.side)
    }

    /// 设置面板尺寸
    pub fn set_size(&mut self, size: f32) {
        self.collapsible_state.set_panel_size(self.side, size);
    }

    /// 设置活动按钮索引
    pub fn set_active_button(&mut self, index: usize) {
        if index < self.buttons.len() {
            self.active_button_index = Some(index);
        }
    }

    /// 获取活动按钮索引
    pub fn get_active_button(&self) -> Option<usize> {
        self.active_button_index
    }

    /// 显示可折叠面板
    pub fn show(&mut self, ctx: &Context, tab_viewer: &mut Tab) -> Option<Response> {
        // 只在第一次调用时从内存加载状态
        if !self.state_loaded {
            let loaded_state = CollapsibleDockState::load_from_memory(ctx, self.state_id);
            if let Some(panel_state) = loaded_state.panels.get(&self.side) {
                if let Some(our_panel_state) = self.collapsible_state.panels.get_mut(&self.side) {
                    our_panel_state.collapsed = panel_state.collapsed;
                    // 验证加载的尺寸是否合理
                    if panel_state.size >= 100.0 {
                        our_panel_state.size = panel_state.size;
                    } else {
                        // 如果加载的尺寸不合理，使用默认值
                        our_panel_state.size = (our_panel_state.min_size * 2.0).max(300.0);
                        println!(
                            "Loaded invalid size {} for panel {:?}, using default {}",
                            panel_state.size, self.side, our_panel_state.size
                        );
                    }
                }
            }
            self.previous_collapsed = self.is_collapsed();
            self.state_loaded = true;
        }

        let is_collapsed = self.is_collapsed();
        self.previous_collapsed = is_collapsed;

        // 如果完全折叠且没有按钮，就不显示面板
        if is_collapsed && self.buttons.is_empty() {
            return None;
        }

        // 创建面板
        let panel_response = match self.side {
            PanelSide::Left => self.show_left_panel(ctx, tab_viewer, is_collapsed),
            PanelSide::Right => self.show_right_panel(ctx, tab_viewer, is_collapsed),
            PanelSide::Top => self.show_top_panel(ctx, tab_viewer, is_collapsed),
            PanelSide::Bottom => self.show_bottom_panel(ctx, tab_viewer, is_collapsed),
        };

        // 保存状态
        self.collapsible_state.save_to_memory(ctx, self.state_id);

        panel_response
    }

    /// 统一的面板渲染方法
    fn show_panel_unified(
        &mut self,
        ctx: &Context,
        tab_viewer: &mut Tab,
        is_collapsed: bool,
    ) -> Option<Response> {
        let side_name = match self.side {
            PanelSide::Left => "left",
            PanelSide::Right => "right",
            PanelSide::Top => "top",
            PanelSide::Bottom => "bottom",
        };

        // 使用更平滑的动画
        let animation_id = self.state_id.with(format!("{}_animation", side_name));
        let target_value = if is_collapsed { 0.0 } else { 1.0 };
        let animation_value = ctx.animate_value_with_time(
            animation_id,
            target_value,
            0.2, // 200ms 的动画时间
        );

        let saved_size = self.get_size();

        // 动态计算折叠宽度：根据图标大小和边距
        let icon_size = 14.0;
        let padding = 6.0; // 左右各3px边距，提供适当的点击区域
        let collapsed_size = icon_size + padding * 2.0; // 26px，更紧凑的设计

        let panel_state = &self.collapsible_state.panels[&self.side];

        // 确保saved_size是合理的，如果不合理则使用默认值
        let validated_saved_size = if saved_size < 100.0 {
            let default_size = (panel_state.min_size * 2.0).max(300.0);
            println!(
                "Invalid saved_size {} for panel {:?}, using default {}",
                saved_size, self.side, default_size
            );
            default_size
        } else {
            saved_size
        };

        // 计算动画中的面板宽度
        let animated_size = if animation_value < 0.01 {
            collapsed_size
        } else if animation_value > 0.99 {
            validated_saved_size
        } else {
            // 使用缓动函数让动画更平滑
            let eased = ease_in_out_cubic(animation_value);
            collapsed_size + (validated_saved_size - collapsed_size) * eased
        };

        // 🔧 关键修复：为展开和折叠状态使用不同的面板ID，避免状态冲突
        let egui_panel_id = if is_collapsed {
            self.state_id.with(format!("{}_collapsed", side_name))
        } else {
            self.state_id.with(format!("{}_expanded", side_name))
        };

        let frame = self.frame.unwrap_or_else(|| {
            let mut frame = Frame::side_top_panel(ctx.style().as_ref());
            frame.stroke = egui::Stroke::NONE;
            frame.inner_margin = egui::Margin::ZERO;
            frame.outer_margin = egui::Margin::ZERO;
            frame
        });

        let panel_response = match self.side {
            PanelSide::Left => {
                // 动态控制resizable：只有在完全展开且用户配置允许时才启用
                let is_resizable = !is_collapsed && panel_state.resizable && animation_value > 0.99;

                let mut panel = egui::SidePanel::left(egui_panel_id)
                    .frame(frame)
                    .show_separator_line(true)
                    .resizable(is_resizable);

                // 根据是否可调整大小设置不同的宽度约束
                if is_resizable {
                    panel = panel
                        .min_width(panel_state.min_size)
                        .max_width(panel_state.max_size.unwrap_or(f32::INFINITY))
                        .default_width(validated_saved_size);
                } else {
                    panel = panel
                        .min_width(animated_size)
                        .max_width(animated_size)
                        .default_width(animated_size);
                }

                panel.show(ctx, |ui| {
                    // 根据动画进度决定显示内容
                    if animation_value < 0.3 {
                        // 折叠状态
                        self.show_collapsed_content(ui, animation_value);
                    } else if animation_value > 0.7 {
                        // 展开状态
                        self.show_expanded_content(ui, tab_viewer);
                    } else {
                        // 过渡状态 - 显示加载或空白
                        ui.centered_and_justified(|ui| {
                            ui.spinner();
                        });
                    }
                })
            }
            PanelSide::Right => {
                // 动态控制resizable：只有在完全展开且用户配置允许时才启用
                let is_resizable = !is_collapsed && panel_state.resizable && animation_value > 0.99;

                let mut panel = egui::SidePanel::right(egui_panel_id)
                    .frame(frame)
                    .show_separator_line(false)
                    .resizable(is_resizable);

                // 根据是否可调整大小设置不同的宽度约束
                if is_resizable {
                    panel = panel
                        .min_width(panel_state.min_size)
                        .max_width(panel_state.max_size.unwrap_or(f32::INFINITY))
                        .default_width(validated_saved_size);
                } else {
                    panel = panel
                        .min_width(animated_size)
                        .max_width(animated_size)
                        .default_width(animated_size);
                }

                panel.show(ctx, |ui| {
                    // 根据动画进度决定显示内容
                    if animation_value < 0.3 {
                        // 折叠状态
                        self.show_collapsed_content(ui, animation_value);
                    } else if animation_value > 0.7 {
                        // 展开状态
                        self.show_expanded_content(ui, tab_viewer);
                    } else {
                        // 过渡状态 - 显示加载或空白
                        ui.centered_and_justified(|ui| {
                            ui.spinner();
                        });
                    }
                })
            }
            PanelSide::Top => {
                // 动态控制resizable：只有在完全展开且用户配置允许时才启用
                let is_resizable = !is_collapsed && panel_state.resizable && animation_value > 0.99;

                let mut panel = egui::TopBottomPanel::top(egui_panel_id)
                    .frame(frame)
                    .show_separator_line(false)
                    .resizable(is_resizable);

                // 根据是否可调整大小设置不同的高度约束
                if is_resizable {
                    panel = panel
                        .min_height(panel_state.min_size)
                        .max_height(panel_state.max_size.unwrap_or(f32::INFINITY))
                        .default_height(validated_saved_size);
                } else {
                    panel = panel
                        .min_height(animated_size)
                        .max_height(animated_size)
                        .default_height(animated_size);
                }

                panel.show(ctx, |ui| {
                    // 根据动画进度决定显示内容
                    if animation_value < 0.3 {
                        // 折叠状态
                        self.show_collapsed_content(ui, animation_value);
                    } else if animation_value > 0.7 {
                        // 展开状态
                        self.show_expanded_content(ui, tab_viewer);
                    } else {
                        // 过渡状态 - 显示加载或空白
                        ui.centered_and_justified(|ui| {
                            ui.spinner();
                        });
                    }
                })
            }
            PanelSide::Bottom => {
                // 动态控制resizable：只有在完全展开且用户配置允许时才启用
                let is_resizable = !is_collapsed && panel_state.resizable && animation_value > 0.99;

                let mut panel = egui::TopBottomPanel::bottom(egui_panel_id)
                    .frame(frame)
                    .show_separator_line(false)
                    .resizable(is_resizable);

                // 根据是否可调整大小设置不同的高度约束
                if is_resizable {
                    panel = panel
                        .min_height(panel_state.min_size)
                        .max_height(panel_state.max_size.unwrap_or(f32::INFINITY))
                        .default_height(validated_saved_size);
                } else {
                    panel = panel
                        .min_height(animated_size)
                        .max_height(animated_size)
                        .default_height(animated_size);
                }

                panel.show(ctx, |ui| {
                    // 根据动画进度决定显示内容
                    if animation_value < 0.3 {
                        // 折叠状态
                        self.show_collapsed_content(ui, animation_value);
                    } else if animation_value > 0.7 {
                        // 展开状态
                        self.show_expanded_content(ui, tab_viewer);
                    } else {
                        // 过渡状态 - 显示加载或空白
                        ui.centered_and_justified(|ui| {
                            ui.spinner();
                        });
                    }
                })
            }
        };

        // 保存用户调整的尺寸
        if !is_collapsed {
            let actual_size = match self.side {
                PanelSide::Left | PanelSide::Right => panel_response.response.rect.width(),
                PanelSide::Top | PanelSide::Bottom => panel_response.response.rect.height(),
            };

            // 只有当尺寸发生显著变化时才保存，避免频繁的微小调整
            let current_saved_size = self.get_size();
            if (actual_size - current_saved_size).abs() > 5.0 {
                // println!("Saving panel size: side={:?}, old={}, new={}",
                //     self.side, current_saved_size, actual_size);
                self.collapsible_state
                    .set_panel_size(self.side, actual_size);
            }
        }

        Some(panel_response.response)
    }

    /// 显示左侧面板
    fn show_left_panel(
        &mut self,
        ctx: &Context,
        tab_viewer: &mut Tab,
        is_collapsed: bool,
    ) -> Option<Response> {
        self.show_panel_unified(ctx, tab_viewer, is_collapsed)
    }

    /// 显示右侧面板
    fn show_right_panel(
        &mut self,
        ctx: &Context,
        tab_viewer: &mut Tab,
        is_collapsed: bool,
    ) -> Option<Response> {
        self.show_panel_unified(ctx, tab_viewer, is_collapsed)
    }

    /// 显示顶部面板
    fn show_top_panel(
        &mut self,
        ctx: &Context,
        tab_viewer: &mut Tab,
        is_collapsed: bool,
    ) -> Option<Response> {
        self.show_panel_unified(ctx, tab_viewer, is_collapsed)
    }

    /// 显示底部面板
    fn show_bottom_panel(
        &mut self,
        ctx: &Context,
        tab_viewer: &mut Tab,
        is_collapsed: bool,
    ) -> Option<Response> {
        self.show_panel_unified(ctx, tab_viewer, is_collapsed)
    }

    /// 显示折叠状态下的内容
    fn show_collapsed_content(&mut self, ui: &mut Ui, animation_value: f32) {
        // 动态计算按钮和图标尺寸，与折叠宽度保持一致
        let icon_size = 14.0; // 图标尺寸
        let padding = 6.0; // 与折叠宽度计算保持一致
        let button_size = Vec2::new(icon_size + padding, icon_size + padding); // 20x20 像素的按钮
        let spacing = 2.0; // 适当的按钮间距

        // 根据面板方向调整布局
        match self.side {
            PanelSide::Left | PanelSide::Right => {
                ui.push_id((self.state_id, "collapsed_vertical"), |ui| {
                    // VS Code 风格的垂直布局
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing.y = spacing;
                            ui.spacing_mut().button_padding = egui::Vec2::ZERO;

                            // 设置背景色
                            let rect = ui.available_rect_before_wrap();
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                ui.style().visuals.extreme_bg_color,
                            );

                            // 显示图标按钮
                            let mut clicked_button = None;
                            for (i, button) in self.buttons.iter().enumerate() {
                                ui.push_id(i, |ui| {
                                    // 折叠状态下，不应该有激活按钮（VS Code 风格）
                                    let is_active =
                                        !self.is_collapsed() && self.active_button_index == Some(i);
                                    let response = self.show_vscode_style_button(
                                        ui,
                                        button,
                                        button_size,
                                        icon_size,
                                        is_active,
                                    );
                                    if response.clicked() {
                                        clicked_button = Some(i);
                                    }
                                });
                            }
                            if let Some(index) = clicked_button {
                                // 展开面板并设置激活按钮
                                self.set_collapsed(false);
                                self.active_button_index = Some(index);
                                // #[cfg(debug_assertions)]
                                // println!("🎯 点击按钮 {} 展开面板，设置为激活状态", index);
                            }
                        },
                    );
                });
            }
            PanelSide::Top | PanelSide::Bottom => {
                ui.push_id((self.state_id, "collapsed_horizontal"), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;

                        // 添加展开按钮
                        if ui
                            .small_button(phosphor::CARET_DOWN)
                            .on_hover_text("展开面板")
                            .clicked()
                        {
                            self.set_collapsed(false);
                        }

                        ui.add_space(4.0);

                        // 显示SVG图标按钮
                        let mut clicked_button = None;
                        for (i, button) in self.buttons.iter().enumerate() {
                            ui.push_id(i, |ui| {
                                let response = self.show_collapsed_svg_button(
                                    ui,
                                    button,
                                    Vec2::splat(icon_size + 4.0), // 为水平布局使用稍小的按钮
                                    animation_value,
                                );
                                if response.clicked() {
                                    clicked_button = Some(i);
                                }
                            });
                        }
                        if let Some(index) = clicked_button {
                            // 展开面板并设置激活按钮
                            self.set_collapsed(false);
                            self.active_button_index = Some(index);
                            // #[cfg(debug_assertions)]
                            // println!("🎯 水平布局：点击按钮 {} 展开面板，设置为激活状态", index);
                        }
                    });
                });
            }
        }
    }

    /// 显示折叠按钮
    fn show_collapsed_button(
        &self,
        ui: &mut Ui,
        button: &CollapsibleButton,
        size: Vec2,
        animation_value: f32,
    ) -> Response {
        let button_text = if let Some(ref icon) = button.icon {
            if animation_value > 0.5 {
                format!("{} {}", icon, button.text)
            } else {
                icon.clone()
            }
        } else {
            if animation_value > 0.5 {
                button.text.clone()
            } else {
                button.text.chars().next().unwrap_or('?').to_string()
            }
        };

        let mut button_ui = egui::Button::new(button_text).min_size(size);

        if button.selected {
            button_ui = button_ui.selected(true);
        }

        let response = ui.add(button_ui);

        // 添加工具提示
        let response = if let Some(ref tooltip) = button.tooltip {
            response.on_hover_text(tooltip)
        } else {
            response.on_hover_text(&button.text)
        };

        response
    }

    /// 显示折叠状态下的Phosphor图标按钮
    fn show_collapsed_svg_button(
        &self,
        ui: &mut Ui,
        button: &CollapsibleButton,
        _size: Vec2,
        _animation_value: f32,
    ) -> Response {
        // 检查是否有 SVG 图标标识符
        if let Some(ref icon_str) = button.icon {
            if icon_str.starts_with("svg:") {
                // 这是一个 SVG 图标，使用自定义渲染
                return self.render_custom_svg_button(ui, button, Vec2::splat(14.0));
                // VSCode style small icon
            }
        }

        // 使用Phosphor图标（原有逻辑）
        let icon = match button.text.as_str() {
            "Search" => phosphor::MAGNIFYING_GLASS,         // 搜索图标
            "Files" => phosphor::FOLDER,                    // 文件夹图标
            "Diagnostics" => phosphor::WARNING,             // 警告图标
            "History" => phosphor::CLOCK_COUNTER_CLOCKWISE, // 历史图标
            "Settings" => phosphor::GEAR,                   // 设置图标
            "场景树" => phosphor::TREE_STRUCTURE,           // 场景树图标
            "属性" => phosphor::LIST_BULLETS,               // 属性图标
            "控制台" => phosphor::TERMINAL,                 // 控制台图标
            _ => phosphor::CIRCLE,                          // 默认圆点
        };

        let button_ui = egui::Button::new(icon).min_size(Vec2::splat(14.0)); // VSCode style small icon
                                                                             // VSCode style: no selection state for collapsed buttons

        let response = ui.add(button_ui);

        // 添加工具提示
        let response = if let Some(ref tooltip) = button.tooltip {
            response.on_hover_text(tooltip)
        } else {
            response.on_hover_text(&button.text)
        };

        response
    }

    /// 显示 VS Code 风格的按钮
    fn show_vscode_style_button(
        &self,
        ui: &mut Ui,
        button: &CollapsibleButton,
        size: Vec2,
        icon_size: f32,
        is_active: bool,
    ) -> Response {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let visuals = ui.style().interact(&response);

            // VS Code 风格的颜色
            let bg_color = if is_active {
                ui.style().visuals.selection.bg_fill
            } else if response.hovered() {
                ui.style().visuals.widgets.hovered.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            };

            let icon_color = if is_active {
                ui.style().visuals.selection.stroke.color
            } else if response.hovered() {
                visuals.text_color()
            } else {
                ui.style().visuals.text_color().gamma_multiply(0.7)
            };

            // 绘制背景
            if bg_color != egui::Color32::TRANSPARENT {
                painter.rect_filled(rect, 0.0, bg_color);
            }

            // 添加活动指示器（左侧或右侧的竖线）
            if is_active {
                let indicator_rect = if self.side == PanelSide::Left {
                    egui::Rect::from_min_size(rect.min, egui::Vec2::new(2.0, rect.height()))
                } else {
                    egui::Rect::from_min_size(
                        egui::Pos2::new(rect.max.x - 2.0, rect.min.y),
                        egui::Vec2::new(2.0, rect.height()),
                    )
                };
                painter.rect_filled(
                    indicator_rect,
                    0.0,
                    ui.style().visuals.selection.stroke.color,
                );
            }

            // 绘制图标
            let icon_rect =
                egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(icon_size));

            // 检查是否有 SVG 图标
            if let Some(ref icon_str) = button.icon {
                if icon_str.starts_with("svg:") {
                    let icon_name = &icon_str[4..];
                    // 调试信息：打印图标名称
                    // #[cfg(debug_assertions)]
                    // println!("🎨 绘制 SVG 图标: {} (来自: {})", icon_name, icon_str);
                    self.draw_custom_svg_icon(ui, icon_name, icon_rect, icon_color);
                } else {
                    // 根据按钮类型绘制不同的图标
                    self.draw_button_icon(painter, &button.text, icon_rect, icon_color, icon_size);
                }
            } else {
                // 根据按钮类型绘制不同的图标
                self.draw_button_icon(painter, &button.text, icon_rect, icon_color, icon_size);
            }
        }

        // 添加工具提示
        if let Some(ref tooltip) = button.tooltip {
            response.on_hover_text(tooltip)
        } else {
            response.on_hover_text(&button.text)
        }
    }

    /// 绘制按钮图标
    fn draw_button_icon(
        &self,
        painter: &egui::Painter,
        button_text: &str,
        rect: egui::Rect,
        color: egui::Color32,
        icon_size: f32,
    ) {
        let stroke = egui::Stroke::new(1.5, color);

        match button_text {
            "场景树" => {
                // 绘制树形结构图标
                let x = rect.left() + rect.width() * 0.2;
                let y_start = rect.top() + rect.height() * 0.2;
                let y_end = rect.bottom() - rect.height() * 0.2;

                // 主干
                painter.line_segment(
                    [egui::Pos2::new(x, y_start), egui::Pos2::new(x, y_end)],
                    stroke,
                );

                // 分支
                for i in 0..3 {
                    let y = y_start + (y_end - y_start) * (i as f32 + 0.5) / 3.0;
                    let x_end = rect.right() - rect.width() * 0.2;
                    painter.line_segment(
                        [
                            egui::Pos2::new(x, y),
                            egui::Pos2::new(x + rect.width() * 0.2, y),
                        ],
                        stroke,
                    );
                    painter.circle_filled(egui::Pos2::new(x_end, y), 2.0, color);
                }
            }
            "属性" => {
                // 绘制列表图标
                for i in 0..3 {
                    let y = rect.top() + rect.height() * (i as f32 + 1.0) / 4.0;
                    let line_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(rect.left() + rect.width() * 0.2, y - 1.0),
                        egui::Vec2::new(rect.width() * 0.6, 2.0),
                    );
                    painter.rect_filled(line_rect, 1.0, color);
                }
            }
            "控制台" => {
                // 绘制终端图标
                let terminal_rect = rect.shrink(rect.width() * 0.15);
                painter.rect_stroke(terminal_rect, 2.0, stroke, egui::StrokeKind::Outside);

                // 提示符
                let prompt_pos = terminal_rect.min
                    + egui::Vec2::new(terminal_rect.width() * 0.1, terminal_rect.height() * 0.6);
                painter.text(
                    prompt_pos,
                    egui::Align2::LEFT_CENTER,
                    ">_",
                    egui::FontId::proportional(icon_size * 0.7),
                    color,
                );
            }
            _ => {
                // 默认图标
                painter.circle_filled(rect.center(), rect.width() * 0.3, color);
            }
        }
    }

    /// 渲染自定义 SVG 按钮
    fn render_custom_svg_button(
        &self,
        ui: &mut Ui,
        button: &CollapsibleButton,
        size: Vec2,
    ) -> Response {
        // 解析图标 ID
        let icon_id = if let Some(ref icon_str) = button.icon {
            if icon_str.starts_with("svg:") {
                let icon_name = &icon_str[4..];
                match icon_name {
                    "SceneTree" => Some("SceneTree"),
                    "Properties" => Some("Properties"),
                    "Console" => Some("Console"),
                    "Files" => Some("Files"),
                    "Terminal" => Some("Terminal"),
                    "Settings" => Some("Settings"),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        // 分配按钮区域
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let text_color = visuals.text_color();
            let bg_fill = visuals.bg_fill;
            let weak_bg_fill = visuals.weak_bg_fill;
            let corner_radius = visuals.corner_radius;
            let bg_stroke = visuals.bg_stroke;

            // 绘制按钮背景
            if button.selected || response.hovered() {
                let bg_color = if button.selected {
                    bg_fill
                } else {
                    weak_bg_fill
                };
                ui.painter().rect_filled(rect, corner_radius, bg_color);
            }

            // 绘制图标
            if let Some(icon_name) = icon_id {
                self.draw_custom_svg_icon(ui, icon_name, rect, text_color);
            } else {
                // 默认图标
                ui.painter()
                    .circle_filled(rect.center(), size.x * 0.3, text_color);
            }

            // 绘制边框
            if response.hovered() || button.selected {
                ui.painter()
                    .rect_stroke(rect, corner_radius, bg_stroke, egui::StrokeKind::Outside);
            }
        }

        // 添加工具提示
        if let Some(ref tooltip) = button.tooltip {
            response.on_hover_text(tooltip)
        } else {
            response.on_hover_text(&button.text)
        }
    }

    /// 绘制自定义 SVG 图标
    fn draw_custom_svg_icon(
        &self,
        ui: &mut Ui,
        icon_name: &str,
        rect: egui::Rect,
        color: egui::Color32,
    ) {
        let painter = ui.painter();
        let center = rect.center();
        let icon_size = rect.size() * 0.8; // 稍微缩小以留出边距
        let icon_rect = egui::Rect::from_center_size(center, icon_size);

        // #[cfg(debug_assertions)]
        // println!("🎨 匹配图标名称: '{}', 可用选项: SceneTree, Properties, Console, Files, Terminal, Settings, Close", icon_name);

        match icon_name {
            "SceneTree" => {
                // #[cfg(debug_assertions)]
                // println!("✅ 绘制场景树图标");
                self.draw_scene_tree_icon(painter, icon_rect, color);
            }
            "Properties" => {
                // #[cfg(debug_assertions)]
                // println!("✅ 绘制属性图标");
                self.draw_properties_icon(painter, icon_rect, color);
            }
            "Console" => {
                // #[cfg(debug_assertions)]
                // println!("✅ 绘制控制台图标");
                self.draw_console_icon(painter, icon_rect, color);
            }
            "Files" => {
                // #[cfg(debug_assertions)]
                // println!("✅ 绘制文件管理器图标");
                self.draw_files_icon(painter, icon_rect, color);
            }
            "Terminal" => {
                // #[cfg(debug_assertions)]
                // println!("✅ 绘制终端图标");
                self.draw_terminal_icon(painter, icon_rect, color);
            }
            "Settings" => {
                // #[cfg(debug_assertions)]
                // println!("✅ 绘制设置图标");
                self.draw_settings_icon(painter, icon_rect, color);
            }
            "Close" => {
                // #[cfg(debug_assertions)]
                // println!("✅ 绘制关闭图标");
                self.draw_close_icon(painter, icon_rect, color);
            }
            _ => {
                // #[cfg(debug_assertions)]
                // println!("❌ 未知图标名称: '{}', 使用默认圆点", icon_name);
                // 默认图标
                painter.circle_filled(center, icon_size.x * 0.3, color);
            }
        }
    }

    /// 绘制场景树图标
    fn draw_scene_tree_icon(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        color: egui::Color32,
    ) {
        let stroke = egui::Stroke::new(1.5, color);
        let line_height = rect.height() / 6.0;
        let indent = rect.width() * 0.15;

        // 垂直连接线
        let x = rect.left() + indent;
        painter.line_segment(
            [
                egui::Pos2::new(x, rect.top() + line_height),
                egui::Pos2::new(x, rect.bottom() - line_height),
            ],
            stroke,
        );

        // 水平线和节点
        for i in 0..3 {
            let y = rect.top() + line_height * (2.0 + i as f32 * 2.0);
            let node_x = x + indent;

            // 水平连接线
            painter.line_segment([egui::Pos2::new(x, y), egui::Pos2::new(node_x, y)], stroke);

            // 节点矩形
            let node_rect = egui::Rect::from_min_size(
                egui::Pos2::new(node_x, y - line_height * 0.3),
                egui::Vec2::new(rect.width() - indent * 2.5, line_height * 0.6),
            );
            painter.rect_filled(node_rect, 2.0, color);
        }
    }

    /// 绘制属性图标
    fn draw_properties_icon(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        color: egui::Color32,
    ) {
        let stroke = egui::Stroke::new(1.5, color);
        let margin = rect.width() * 0.1;
        let content_rect = rect.shrink(margin);

        // 外框
        painter.rect_stroke(content_rect, 3.0, stroke, egui::StrokeKind::Outside);

        // 内容线条
        let line_height = content_rect.height() / 6.0;
        for i in 0..3 {
            let y = content_rect.top() + line_height * (1.5 + i as f32 * 1.5);
            let line_width = content_rect.width() * (0.8 - i as f32 * 0.1);
            let line_rect = egui::Rect::from_min_size(
                egui::Pos2::new(content_rect.left() + margin, y - 1.0),
                egui::Vec2::new(line_width, 2.0),
            );
            painter.rect_filled(line_rect, 1.0, color);
        }
    }

    /// 绘制控制台图标
    fn draw_console_icon(&self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
        let stroke = egui::Stroke::new(1.5, color);
        let margin = rect.width() * 0.05;
        let console_rect = rect.shrink(margin);

        // 控制台外框
        painter.rect_stroke(console_rect, 3.0, stroke, egui::StrokeKind::Outside);

        // 命令提示符 ">"
        let prompt_size = console_rect.width() * 0.15;
        let prompt_center =
            egui::Pos2::new(console_rect.left() + prompt_size, console_rect.center().y);

        // 绘制三角形提示符
        let triangle_points = [
            egui::Pos2::new(
                prompt_center.x - prompt_size * 0.3,
                prompt_center.y - prompt_size * 0.3,
            ),
            egui::Pos2::new(prompt_center.x + prompt_size * 0.3, prompt_center.y),
            egui::Pos2::new(
                prompt_center.x - prompt_size * 0.3,
                prompt_center.y + prompt_size * 0.3,
            ),
        ];
        painter.add(egui::Shape::convex_polygon(
            triangle_points.to_vec(),
            color,
            egui::Stroke::NONE,
        ));

        // 命令行
        let line_rect = egui::Rect::from_min_size(
            egui::Pos2::new(prompt_center.x + prompt_size, prompt_center.y - 1.0),
            egui::Vec2::new(console_rect.width() * 0.5, 2.0),
        );
        painter.rect_filled(line_rect, 1.0, color);
    }

    /// 绘制文件管理器图标 (Files)
    fn draw_files_icon(&self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
        let stroke = egui::Stroke::new(1.5, color);

        // 后面的文件夹
        let back_folder = egui::Rect::from_min_size(
            rect.min + egui::Vec2::new(rect.width() * 0.1, rect.height() * 0.3),
            egui::Vec2::new(rect.width() * 0.6, rect.height() * 0.5),
        );
        painter.rect_stroke(back_folder, 2.0, stroke, egui::StrokeKind::Outside);

        // 前面的文件夹
        let front_folder = egui::Rect::from_min_size(
            rect.min + egui::Vec2::new(rect.width() * 0.3, rect.height() * 0.15),
            egui::Vec2::new(rect.width() * 0.6, rect.height() * 0.5),
        );
        painter.rect_filled(front_folder, 2.0, color.gamma_multiply(0.1));
        painter.rect_stroke(front_folder, 2.0, stroke, egui::StrokeKind::Outside);

        // 文件夹标签
        let tab_rect = egui::Rect::from_min_size(
            front_folder.min - egui::Vec2::new(0.0, rect.height() * 0.08),
            egui::Vec2::new(rect.width() * 0.25, rect.height() * 0.08),
        );
        painter.rect_filled(tab_rect, 1.0, color.gamma_multiply(0.15));
    }

    /// 绘制终端图标 (Terminal)
    fn draw_terminal_icon(&self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
        let stroke = egui::Stroke::new(1.5, color);

        // 终端窗口边框
        let terminal_rect = rect.shrink(rect.width() * 0.1);
        painter.rect_stroke(terminal_rect, 3.0, stroke, egui::StrokeKind::Outside);

        // 命令提示符 ">"
        let prompt_center = egui::Pos2::new(
            terminal_rect.left() + terminal_rect.width() * 0.2,
            terminal_rect.center().y,
        );

        // 绘制 ">" 符号
        painter.line_segment(
            [
                egui::Pos2::new(prompt_center.x - 5.0, prompt_center.y - 5.0),
                egui::Pos2::new(prompt_center.x, prompt_center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                egui::Pos2::new(prompt_center.x - 5.0, prompt_center.y + 5.0),
                egui::Pos2::new(prompt_center.x, prompt_center.y),
            ],
            stroke,
        );

        // 光标
        let cursor_rect = egui::Rect::from_min_size(
            egui::Pos2::new(prompt_center.x + 10.0, prompt_center.y - 1.0),
            egui::Vec2::new(8.0, 2.0),
        );
        painter.rect_filled(cursor_rect, 0.0, color);
    }

    /// 绘制设置图标 (Settings)
    fn draw_settings_icon(&self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.35;

        // 绘制齿轮形状（简化版）
        // 中心圆
        painter.circle_filled(center, radius * 0.4, color.gamma_multiply(0.1));
        painter.circle_stroke(center, radius * 0.4, egui::Stroke::new(1.5, color));

        // 齿轮齿
        let teeth_count = 8;
        for i in 0..teeth_count {
            let angle = (i as f32) * std::f32::consts::TAU / (teeth_count as f32);
            let tooth_inner = center + egui::Vec2::angled(angle) * (radius * 0.5);
            let tooth_outer = center + egui::Vec2::angled(angle) * radius;

            painter.line_segment([tooth_inner, tooth_outer], egui::Stroke::new(2.0, color));
        }
    }

    /// 绘制关闭图标 (Close)
    fn draw_close_icon(&self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
        let stroke = egui::Stroke::new(2.0, color);
        let center = rect.center();
        let size = rect.width().min(rect.height()) * 0.4;

        // 绘制 X 符号的两条对角线
        let half_size = size * 0.5;

        // 左上到右下的对角线
        painter.line_segment(
            [
                egui::Pos2::new(center.x - half_size, center.y - half_size),
                egui::Pos2::new(center.x + half_size, center.y + half_size),
            ],
            stroke,
        );

        // 右上到左下的对角线
        painter.line_segment(
            [
                egui::Pos2::new(center.x + half_size, center.y - half_size),
                egui::Pos2::new(center.x - half_size, center.y + half_size),
            ],
            stroke,
        );
    }

    /// 显示展开状态下的内容
    fn show_expanded_content(&mut self, ui: &mut Ui, tab_viewer: &mut Tab) {
        // 去掉上方的最小化按钮，直接显示 dock 内容
        // 显示 dock 内容，使用唯一的 ID
        ui.push_id((self.state_id, "dock_area"), |ui| {
            egui_dock::DockArea::new(&mut self.dock_state)
                .id(egui::Id::new((self.state_id, "dock_area_unique")))
                .style(egui_dock::Style::from_egui(ui.ctx().style().as_ref()))
                .show_leaf_collapse_buttons(false) // 直接禁用 collapse 按钮
                .show_close_buttons(true) // 启用关闭按钮，但功能改为最小化面板
                .show_add_buttons(false) // 禁用添加按钮
                .show_inside(ui, tab_viewer);
        });
    }
}
