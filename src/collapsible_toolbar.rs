use egui::{Context, Frame, Id, Response, Ui};
use serde::{Deserialize, Serialize};

/// 面板方向枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// 标签页特征，定义标签页的基本行为
pub trait TabViewer {
    type Tab: Clone + PartialEq + Serialize + for<'de> Deserialize<'de>;

    /// 获取标签页的标题
    fn title(&self, tab: &Self::Tab) -> String;

    /// 渲染标签页的内容
    fn ui(&mut self, ui: &mut Ui, tab: &Self::Tab);

    /// 标签页是否可以关闭
    fn closable(&self, _tab: &Self::Tab) -> bool {
        false
    }
}

/// 工具栏状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarState<Tab> {
    /// 所有标签页
    pub tabs: Vec<Tab>,
    /// 当前选中的标签页索引
    pub selected_tab: Option<usize>,
    /// 是否展开
    pub is_expanded: bool,
}

impl<Tab> Default for ToolbarState<Tab> {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            selected_tab: None,
            is_expanded: false,
        }
    }
}

/// 可折叠工具栏组件
pub struct CollapsibleToolbar<Tab> {
    /// 面板方向
    side: PanelSide,
    /// 默认标签页
    default_tabs: Vec<Tab>,
    /// 是否启用状态持久化
    persist: bool,
    /// 展开时的框架样式
    expanded_frame: Option<Frame>,
    /// 标签页框架样式
    tabs_frame: Option<Frame>,
    /// 最小宽度/高度
    min_size: f32,
    /// 是否可调整大小
    resizable: bool,
}

impl<Tab> CollapsibleToolbar<Tab>
where
    Tab: Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static + Send + Sync,
{
    /// 创建新的可折叠工具栏
    pub fn new(side: PanelSide, default_tabs: Vec<Tab>) -> Self {
        Self {
            side,
            default_tabs,
            persist: false,
            expanded_frame: None,
            tabs_frame: None,
            min_size: 200.0,
            resizable: true,
        }
    }

    /// 设置是否启用状态持久化
    pub fn persist(mut self, persist: bool) -> Self {
        self.persist = persist;
        self
    }

    /// 设置展开时的框架样式
    pub fn expanded_frame(mut self, frame: Frame) -> Self {
        self.expanded_frame = Some(frame);
        self
    }

    /// 设置标签页框架样式
    pub fn tabs_frame(mut self, frame: Frame) -> Self {
        self.tabs_frame = Some(frame);
        self
    }

    /// 设置最小尺寸
    pub fn min_size(mut self, size: f32) -> Self {
        self.min_size = size;
        self
    }

    /// 设置是否可调整大小
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// 显示工具栏
    pub fn show(
        &self,
        ctx: &Context,
        id: impl Into<Id>,
        viewer: &mut impl TabViewer<Tab = Tab>,
    ) -> Option<Response> {
        let id = id.into();
        let state_id = id.with("toolbar_state");

        // 加载状态
        let mut state = self.load_state(ctx, state_id);

        // 如果状态为空，使用默认标签页
        if state.tabs.is_empty() {
            state.tabs = self.default_tabs.clone();
        }

        // 渲染工具栏
        let response = self.show_toolbar(ctx, id, &mut state, viewer);

        // 保存状态
        self.save_state(ctx, state_id, &state);

        response
    }

    /// 加载工具栏状态
    fn load_state(&self, ctx: &Context, state_id: Id) -> ToolbarState<Tab> {
        ctx.memory_mut(|mem| {
            let default_state = || ToolbarState {
                tabs: self.default_tabs.clone(),
                selected_tab: None,
                is_expanded: false,
            };

            if self.persist {
                mem.data
                    .get_persisted_mut_or_insert_with(state_id, default_state)
                    .clone()
            } else {
                mem.data
                    .get_temp_mut_or_insert_with(state_id, default_state)
                    .clone()
            }
        })
    }

    /// 保存工具栏状态
    fn save_state(&self, ctx: &Context, state_id: Id, state: &ToolbarState<Tab>) {
        ctx.memory_mut(|mem| {
            if self.persist {
                mem.data.insert_persisted(state_id, state.clone());
            } else {
                mem.data.insert_temp(state_id, state.clone());
            }
        });
    }

    /// 渲染工具栏界面
    fn show_toolbar(
        &self,
        ctx: &Context,
        id: Id,
        state: &mut ToolbarState<Tab>,
        viewer: &mut impl TabViewer<Tab = Tab>,
    ) -> Option<Response> {
        let animation_time = 0.2; // 动画持续时间（秒）

        // 根据面板方向创建相应的面板
        match self.side {
            PanelSide::Left => {
                let collapsed_width = 16.0;  // VSCode style narrow sidebar
                let expanded_width = self.min_size;

                egui::SidePanel::left(id)
                    .frame(self.expanded_frame.unwrap_or_else(|| {
                        let mut frame = Frame::side_top_panel(&egui::Style::default());
                        // Remove all padding for VSCode-style collapsed width
                        if !state.is_expanded {
                            frame.inner_margin = egui::Margin::ZERO;
                            frame.outer_margin = egui::Margin::ZERO;
                        }
                        frame
                    }))
                    .min_width(collapsed_width)
                    .default_width(if state.is_expanded { expanded_width } else { collapsed_width })
                    .width_range(collapsed_width..=expanded_width * 2.0)
                    .resizable(self.resizable && state.is_expanded)
                    .show_animated(ctx, state.is_expanded, |ui| {
                        self.show_content(ui, state, viewer)
                    })
                    .map(|r| r.response)
            }
            PanelSide::Right => {
                let collapsed_width = 16.0;  // VSCode style narrow sidebar
                let expanded_width = self.min_size;

                egui::SidePanel::right(id)
                    .frame(self.expanded_frame.unwrap_or_else(|| {
                        let mut frame = Frame::side_top_panel(&egui::Style::default());
                        // Remove all padding for VSCode-style collapsed width
                        if !state.is_expanded {
                            frame.inner_margin = egui::Margin::ZERO;
                            frame.outer_margin = egui::Margin::ZERO;
                        }
                        frame
                    }))
                    .min_width(collapsed_width)
                    .default_width(if state.is_expanded { expanded_width } else { collapsed_width })
                    .width_range(collapsed_width..=expanded_width * 2.0)
                    .resizable(self.resizable && state.is_expanded)
                    .show_animated(ctx, state.is_expanded, |ui| {
                        self.show_content(ui, state, viewer)
                    })
                    .map(|r| r.response)
            }
            PanelSide::Top => {
                let collapsed_height = 35.0;
                let expanded_height = self.min_size;

                egui::TopBottomPanel::top(id)
                    .frame(self.expanded_frame.unwrap_or_else(|| Frame::side_top_panel(&egui::Style::default())))
                    .min_height(collapsed_height)
                    .default_height(if state.is_expanded { expanded_height } else { collapsed_height })
                    .height_range(collapsed_height..=expanded_height * 2.0)
                    .resizable(self.resizable && state.is_expanded)
                    .show_animated(ctx, state.is_expanded, |ui| {
                        self.show_content(ui, state, viewer)
                    })
                    .map(|r| r.response)
            }
            PanelSide::Bottom => {
                let collapsed_height = 35.0;
                let expanded_height = self.min_size;

                egui::TopBottomPanel::bottom(id)
                    .frame(self.expanded_frame.unwrap_or_else(|| Frame::side_top_panel(&egui::Style::default())))
                    .min_height(collapsed_height)
                    .default_height(if state.is_expanded { expanded_height } else { collapsed_height })
                    .height_range(collapsed_height..=expanded_height * 2.0)
                    .resizable(self.resizable && state.is_expanded)
                    .show_animated(ctx, state.is_expanded, |ui| {
                        self.show_content(ui, state, viewer)
                    })
                    .map(|r| r.response)
            }
        }
    }

    /// 显示工具栏内容
    fn show_content(
        &self,
        ui: &mut Ui,
        state: &mut ToolbarState<Tab>,
        viewer: &mut impl TabViewer<Tab = Tab>,
    ) {
        if state.is_expanded {
            // 展开状态：显示标签页和内容
            self.show_expanded_content(ui, state, viewer);
        } else {
            // 收叠状态：只显示标签页按钮
            self.show_collapsed_tabs(ui, state, viewer);
        }
    }

    /// 显示展开状态的内容
    fn show_expanded_content(
        &self,
        ui: &mut Ui,
        state: &mut ToolbarState<Tab>,
        viewer: &mut impl TabViewer<Tab = Tab>,
    ) {
        // 标签页栏
        let _tabs_response = self.show_tab_bar(ui, state, viewer);

        // 内容区域
        if let Some(selected_idx) = state.selected_tab {
            if let Some(selected_tab) = state.tabs.get(selected_idx) {
                ui.separator();
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        viewer.ui(ui, selected_tab);
                    });
            }
        }
    }

    /// 显示收叠状态的标签页
    fn show_collapsed_tabs(
        &self,
        ui: &mut Ui,
        state: &mut ToolbarState<Tab>,
        viewer: &mut impl TabViewer<Tab = Tab>,
    ) {
        let is_vertical = matches!(self.side, PanelSide::Left | PanelSide::Right);

        if is_vertical {
            // 垂直排列的标签页按钮
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for (idx, tab) in state.tabs.iter().enumerate() {
                    let is_selected = state.selected_tab == Some(idx);

                    // 提取标签页标题的第一个字符或图标
                    let title = viewer.title(tab);
                    let short_title = if let Some(icon_end) = title.find(' ') {
                        &title[..icon_end] // 只显示图标部分
                    } else {
                        &title[..title.len().min(2)] // 或者前两个字符
                    };

                    let button = egui::Button::new(short_title)
                        // VSCode style: no selection state when collapsed
                        .min_size(egui::vec2(14.0, 14.0));

                    if ui.add(button).on_hover_text(&title).clicked() {
                        state.selected_tab = Some(idx);
                        state.is_expanded = true;
                    }
                }
            });
        } else {
            // 水平排列的标签页按钮
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (idx, tab) in state.tabs.iter().enumerate() {
                    let is_selected = state.selected_tab == Some(idx);

                    // 提取标签页标题的第一个字符或图标
                    let title = viewer.title(tab);
                    let short_title = if let Some(icon_end) = title.find(' ') {
                        &title[..icon_end] // 只显示图标部分
                    } else {
                        &title[..title.len().min(2)] // 或者前两个字符
                    };

                    let button = egui::Button::new(short_title)
                        // VSCode style: no selection state when collapsed
                        .min_size(egui::vec2(14.0, 14.0));

                    if ui.add(button).on_hover_text(&title).clicked() {
                        state.selected_tab = Some(idx);
                        state.is_expanded = true;
                    }
                }
            });
        }
    }

    /// 显示标签页栏
    fn show_tab_bar(
        &self,
        ui: &mut Ui,
        state: &mut ToolbarState<Tab>,
        viewer: &mut impl TabViewer<Tab = Tab>,
    ) -> Response {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;

            // 标签页按钮
            for (idx, tab) in state.tabs.iter().enumerate() {
                let is_selected = state.selected_tab == Some(idx);

                // 创建带有样式的标签页按钮
                let button = egui::Button::new(viewer.title(tab))
                    .selected(is_selected)
                    .corner_radius(4.0);

                let response = ui.add(button);

                if response.clicked() {
                    if is_selected {
                        // 点击当前选中的标签页，收叠工具栏
                        state.selected_tab = None;
                        state.is_expanded = false;
                    } else {
                        // 选中新的标签页
                        state.selected_tab = Some(idx);
                        state.is_expanded = true;
                    }
                }

                // 右键菜单（如果标签页可关闭）
                if viewer.closable(tab) {
                    response.context_menu(|ui| {
                        if ui.button("关闭标签页").clicked() {
                            // TODO: 实现关闭标签页的逻辑
                            ui.close();
                        }
                    });
                }
            }

            // 添加弹性空间
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 收叠按钮
                let close_button = egui::Button::new("✕")
                    .small()
                    .corner_radius(2.0);

                if ui.add(close_button).on_hover_text("收叠工具栏").clicked() {
                    state.is_expanded = false;
                    state.selected_tab = None;
                }
            });
        })
        .response
    }
}
