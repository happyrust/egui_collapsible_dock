use egui::{Context, Frame, Id, Response, Ui, Vec2};
use egui_dock::{DockState, TabViewer};
use egui_phosphor::regular as phosphor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            size: 250.0,
            min_size: 150.0, // 恢复原来的最小尺寸
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
            let old_size = panel.size;
            // 不受min_size限制，直接保存用户调整的实际宽度
            panel.size = size;
            if let Some(max_size) = panel.max_size {
                panel.size = panel.size.min(max_size);
            }
        } else {
            println!("set_panel_size: panel not found for side={:?}", side);
        }
    }

    /// 获取面板尺寸
    pub fn get_panel_size(&self, side: PanelSide) -> f32 {
        self.panels.get(&side).map(|p| p.size).unwrap_or(PanelState::default().size)
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

    /// 显示可折叠面板
    pub fn show(&mut self, ctx: &Context, tab_viewer: &mut Tab) -> Option<Response> {
        // 只在第一次调用时从内存加载状态
        if !self.state_loaded {
            let loaded_state = CollapsibleDockState::load_from_memory(ctx, self.state_id);
            if let Some(panel_state) = loaded_state.panels.get(&self.side) {
                if let Some(our_panel_state) = self.collapsible_state.panels.get_mut(&self.side) {
                    our_panel_state.collapsed = panel_state.collapsed;
                    our_panel_state.size = panel_state.size;
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

        let animation_value = ctx.animate_bool(
            self.state_id.with(format!("{}_animation", side_name)),
            !is_collapsed
        );

        let saved_size = self.get_size();
        let collapsed_size = 40.0;
        let panel_state = &self.collapsible_state.panels[&self.side];

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
                egui::SidePanel::left(egui_panel_id)
                    .frame(frame)
                    .show_separator_line(true)
                    .min_width(if is_collapsed { collapsed_size } else { panel_state.min_size })
                    .max_width(if is_collapsed { collapsed_size } else { f32::INFINITY })
                    .default_width(if is_collapsed { collapsed_size } else { saved_size })
                    .resizable(!is_collapsed && panel_state.resizable)
                    .show(ctx, |ui| {
                        if is_collapsed {
                            self.show_collapsed_content(ui, animation_value);
                        } else {
                            self.show_expanded_content(ui, tab_viewer);
                        }
                    })
            }
            PanelSide::Right => {
                egui::SidePanel::right(egui_panel_id)
                    .frame(frame)
                    .show_separator_line(false)
                    .min_width(if is_collapsed { collapsed_size } else { panel_state.min_size })
                    .max_width(if is_collapsed { collapsed_size } else { f32::INFINITY })
                    .default_width(if is_collapsed { collapsed_size } else { saved_size })
                    .resizable(!is_collapsed && panel_state.resizable)
                    .show(ctx, |ui| {
                        if is_collapsed {
                            self.show_collapsed_content(ui, animation_value);
                        } else {
                            self.show_expanded_content(ui, tab_viewer);
                        }
                    })
            }
            PanelSide::Top => {
                egui::TopBottomPanel::top(egui_panel_id)
                    .frame(frame)
                    .show_separator_line(false)
                    .min_height(if is_collapsed { collapsed_size } else { panel_state.min_size })
                    .max_height(if is_collapsed {
                        collapsed_size
                    } else {
                        panel_state.max_size.unwrap_or(f32::INFINITY)
                    })
                    .default_height(if is_collapsed { collapsed_size } else { saved_size })
                    .resizable(!is_collapsed && panel_state.resizable)
                    .show(ctx, |ui| {
                        if is_collapsed {
                            self.show_collapsed_content(ui, animation_value);
                        } else {
                            self.show_expanded_content(ui, tab_viewer);
                        }
                    })
            }
            PanelSide::Bottom => {
                egui::TopBottomPanel::bottom(egui_panel_id)
                    .frame(frame)
                    .show_separator_line(false)
                    .min_height(if is_collapsed { collapsed_size } else { panel_state.min_size })
                    .max_height(if is_collapsed {
                        collapsed_size
                    } else {
                        panel_state.max_size.unwrap_or(f32::INFINITY)
                    })
                    .default_height(if is_collapsed { collapsed_size } else { saved_size })
                    .resizable(!is_collapsed && panel_state.resizable)
                    .show(ctx, |ui| {
                        if is_collapsed {
                            self.show_collapsed_content(ui, animation_value);
                        } else {
                            self.show_expanded_content(ui, tab_viewer);
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
            self.collapsible_state.set_panel_size(self.side, actual_size);
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
        // 按钮大小应该匹配折叠面板的宽度
        let button_size = Vec2::splat(40.0);
        let spacing = 2.0; // 紧凑但有适当间距

        // 根据面板方向调整布局
        match self.side {
            PanelSide::Left | PanelSide::Right => {
                ui.push_id((self.state_id, "collapsed_vertical"), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.spacing_mut().item_spacing.y = spacing;

                        // 添加展开按钮
                        if ui
                            .small_button(phosphor::CARET_RIGHT)
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
                                    button_size,
                                    animation_value,
                                );
                                if response.clicked() {
                                    clicked_button = Some(i);
                                }
                            });
                        }
                        if clicked_button.is_some() {
                            self.set_collapsed(false);
                        }
                    });
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
                                    button_size,
                                    animation_value,
                                );
                                if response.clicked() {
                                    clicked_button = Some(i);
                                }
                            });
                        }
                        if clicked_button.is_some() {
                            self.set_collapsed(false);
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
        // 使用Phosphor图标
        let icon = match button.text.as_str() {
            "Search" => phosphor::MAGNIFYING_GLASS,         // 搜索图标
            "Files" => phosphor::FOLDER,                    // 文件夹图标
            "Diagnostics" => phosphor::WARNING,             // 警告图标
            "History" => phosphor::CLOCK_COUNTER_CLOCKWISE, // 历史图标
            "Settings" => phosphor::GEAR,                   // 设置图标
            _ => phosphor::CIRCLE,                          // 默认圆点
        };

        let mut button_ui = egui::Button::new(icon).min_size(Vec2::splat(28.0));

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

    /// 显示展开状态下的内容
    fn show_expanded_content(&mut self, ui: &mut Ui, tab_viewer: &mut Tab) {
        // 显示紧凑的标题栏和最小化按钮
        ui.push_id((self.state_id, "expanded_header"), |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.push_id("minimize_button", |ui| {
                        // 使用Phosphor图标作为最小化按钮
                        if ui
                            .small_button(phosphor::MINUS)
                            .on_hover_text("最小化面板")
                            .clicked()
                        {
                            self.set_collapsed(true);
                        }
                    });
                });
            });
        });

        // 显示 dock 内容，使用唯一的 ID
        ui.push_id((self.state_id, "dock_area"), |ui| {
            egui_dock::DockArea::new(&mut self.dock_state)
                .id(egui::Id::new((self.state_id, "dock_area_unique")))
                .style(egui_dock::Style::from_egui(ui.ctx().style().as_ref()))
                .show_leaf_collapse_buttons(false) // 直接禁用 collapse 按钮
                .show_inside(ui, tab_viewer);
        });
    }
}
