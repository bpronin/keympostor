use super::*;
use crate::res::res_ids::{IDI_ICON_APP, IDS_APP_TITLE, IDS_LAYOUT, IDS_LOG};
use crate::res::RESOURCES;
use crate::ui::style::INFO_LABEL_FONT;
use crate::{r_icon, rs};
use keympostor::layout::Layout;
use native_windows_gui as nwg;
use native_windows_gui::stretch::geometry::Rect;
use native_windows_gui::stretch::style::Dimension;
use native_windows_gui::{ControlHandle, NwgError};
use nwg::stretch::style::Dimension::Points as PT;

#[derive(Default)]
pub(crate) struct MainWindow {
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    tab_log_layout: nwg::FlexboxLayout,
    tab_layouts_layout: nwg::FlexboxLayout,
    tab_log: nwg::Tab,
    tab_layouts: nwg::Tab,
    main_menu: MainMenu,
    tab_container: nwg::TabsContainer,
    layout_view: LayoutView,
    log_view: LogView,
    key_event_label: nwg::Label,
    test_editor: TypeTestEditor,
    tray: Tray,
}

impl MainWindow {
    pub(crate) fn build(&mut self) -> Result<(), NwgError> {
        nwg::Window::builder()
            .size((700, 300))
            .icon(Some(r_icon!(IDI_ICON_APP)))
            .flags(nwg::WindowFlags::MAIN_WINDOW)
            .title(rs!(IDS_APP_TITLE))
            .build(&mut self.window)?;

        nwg::Label::builder()
            .parent(&self.window)
            .text("*")
            .font(Some(&INFO_LABEL_FONT))
            .build(&mut self.key_event_label)?;

        self.test_editor.build(&mut self.window)?;

        /* Tabs */

        nwg::TabsContainer::builder()
            .parent(&self.window)
            .build(&mut self.tab_container)?;

        nwg::Tab::builder()
            .text(rs!(IDS_LOG))
            .parent(&self.tab_container)
            .build(&mut self.tab_log)?;

        nwg::Tab::builder()
            .text(rs!(IDS_LAYOUT))
            .parent(&self.tab_container)
            .build(&mut self.tab_layouts)?;

        self.main_menu.build(&mut self.window)?;
        self.log_view.build(&mut self.tab_log)?;
        self.layout_view.build(&mut self.tab_layouts)?;
        self.tray.build(&self.window)?;

        self.layout()
    }

    fn layout(&self) -> Result<(), nwg::NwgError> {
        use nwg::stretch::{geometry::Size, style::FlexDirection};

        /* Log tab layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.tab_container)
            // .padding(TAB_PADDING)
            .child(self.log_view.view())
            .child_margin(Rect {
                start: PT(4.0),
                end: PT(16.0),
                top: PT(6.0),
                bottom: PT(40.0),
            })
            .build(&self.tab_log_layout)?;

        /* Layout tab layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.tab_container)
            // .padding(TAB_PADDING)
            .child(self.layout_view.view())
            .child_margin(Rect {
                start: PT(4.0),
                end: PT(16.0),
                top: PT(6.0),
                bottom: PT(40.0),
            })
            .build(&self.tab_layouts_layout)?;

        /* Main window layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.window)
            .flex_direction(FlexDirection::Column)
            // .padding(PADDING)
            /* Tabs */
            .child(&self.tab_container)
            // .child_margin(MARGIN)
            .child_flex_grow(1.0)
            /* Test label */
            .child(&self.key_event_label)
            // .child_margin(MARGIN_2)
            .child_size(Size {
                width: Dimension::Auto,
                height: Dimension::Points(24.0),
            })
            /* Test editor */
            .child(self.test_editor.editor())
            // .child_margin(MARGIN_2)
            .child_size(Size {
                width: Dimension::Auto,
                height: Dimension::Points(40.0),
            })
            .build(&self.layout)
    }

    pub(crate) fn handle_event(&self, app: &App, evt: nwg::Event, handle: nwg::ControlHandle) {
        self.main_menu.handle_event(app, evt, handle);
        self.tray.handle_event(app, evt, handle);
        self.test_editor.handle_event(evt);
        match evt {
            nwg::Event::OnWindowClose => {
                if &handle == &self.window.handle {
                    app.on_window_close()
                }
            }
            _ => {}
        }
    }

    pub(crate) fn update_ui(
        &self,
        is_processing_enabled: bool,
        is_auto_switch_layout_enabled: bool,
        is_logging_enabled: bool,
        current_layout_name: &Option<String>,
    ) {
        self.main_menu.update_ui(
            is_processing_enabled,
            is_auto_switch_layout_enabled,
            is_logging_enabled,
            current_layout_name,
        );

        self.tray.update_ui(is_processing_enabled);
    }

    pub(crate) fn handle(&self) -> ControlHandle {
        self.window.handle
    }

    pub(crate) fn set_title(&self, title: &str) {
        self.window.set_text(title)
    }

    pub(crate) fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible);
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.window.visible()
    }

    pub(crate) fn set_size(&self, size: Option<(u32, u32)>) {
        if let Some(size) = size {
            set_window_size(self.window.handle, size);
        }
    }

    pub(crate) fn get_size(&self) -> (u32, u32) {
        get_window_size(self.window.handle)
    }

    pub(crate) fn set_position(&self, position: Option<(i32, i32)>) {
        if let Some(position) = position {
            self.window.set_position(position.0, position.1)
        }
    }

    pub(crate) fn get_position(&self) -> (i32, i32) {
        self.window.position()
    }

    pub(crate) fn set_layouts(&self, layouts: &Layouts) {
        self.main_menu.build_layouts_menu(layouts);
    }

    pub(crate) fn on_select_layout(&self, layout: &Layout) {
        self.layout_view.update_ui(layout);
    }

    pub(crate) fn on_key_hook_notify(&self, event: &KeyEvent) {
        self.log_view.append(event);
        self.key_event_label
            .set_text(KeyTrigger::from(event).to_string().as_str());
    }

    pub(crate) fn clear_log(&self) {
        self.log_view.clear()
    }

    pub(crate) fn get_selected_page(&self) -> usize {
        self.tab_container.selected_tab()
    }

    pub(crate) fn set_selected_page(&self, page: Option<usize>) {
        if let Some(page) = page {
            self.tab_container.set_selected_tab(page);
        }
    }
}
