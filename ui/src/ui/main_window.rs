use crate::app::App;
use crate::layout::{KeyTransformLayout, KeyTransformLayouts};
use crate::settings::MainWindowSettings;
use crate::ui::layout_view::LayoutView;
use crate::ui::log_view::LogView;
use crate::ui::main_menu::MainMenu;
use crate::ui::res::RESOURCES;
use crate::ui::res_ids::{IDI_ICON_APP, IDS_APP_TITLE, IDS_LAYOUT, IDS_LOG};
use crate::ui::style::INFO_LABEL_FONT;
use crate::ui::test_editor::TypeTestEditor;
use crate::ui::tray::Tray;
use crate::ui::utils::hwnd;
use crate::{r_icon, rs, ui};
use keympostor::event::KeyEvent;
use native_windows_gui::stretch::geometry::{Rect, Size};
use native_windows_gui::stretch::style::Dimension::Points as PT;
use native_windows_gui::stretch::style::{Dimension as D, FlexDirection};
use native_windows_gui::{
    ControlHandle, Event, FlexboxLayout, Label, NwgError, Tab, TabsContainer, Window, WindowFlags,
};
use windows::Win32::Foundation::HWND;

#[derive(Default)]
pub(crate) struct MainWindow {
    window: Window,
    layout: FlexboxLayout,
    tab_log_layout: FlexboxLayout,
    tab_layouts_layout: FlexboxLayout,
    tab_log: Tab,
    tab_layouts: Tab,
    main_menu: MainMenu,
    tab_container: TabsContainer,
    layout_view: LayoutView,
    log_view: LogView,
    key_event_label: Label,
    test_editor: TypeTestEditor,
    tray: Tray,
}

impl MainWindow {
    pub(crate) fn build(&mut self) -> Result<(), NwgError> {
        Window::builder()
            .size((700, 300))
            .icon(Some(&r_icon!(IDI_ICON_APP)))
            .flags(WindowFlags::MAIN_WINDOW)
            .title(rs!(IDS_APP_TITLE))
            .build(&mut self.window)?;

        Label::builder()
            .parent(&self.window)
            .text("*")
            .font(Some(&INFO_LABEL_FONT))
            .build(&mut self.key_event_label)?;

        self.test_editor.build(&mut self.window)?;

        /* Tabs */

        TabsContainer::builder()
            .parent(&self.window)
            .build(&mut self.tab_container)?;

        Tab::builder()
            .text(rs!(IDS_LOG))
            .parent(&self.tab_container)
            .build(&mut self.tab_log)?;

        Tab::builder()
            .text(rs!(IDS_LAYOUT))
            .parent(&self.tab_container)
            .build(&mut self.tab_layouts)?;

        self.main_menu.build(&mut self.window)?;
        self.log_view.build(&mut self.tab_log)?;
        self.layout_view.build(&mut self.tab_layouts)?;
        self.tray.build(&self.window)?;

        self.layout()
    }

    fn layout(&self) -> Result<(), NwgError> {
        /* Log tab */
        FlexboxLayout::builder()
            .parent(&self.tab_container)
            .child(self.log_view.view())
            .child_margin(Rect {
                start: PT(4.0),
                end: PT(16.0),
                top: PT(6.0),
                bottom: PT(40.0),
            })
            .build(&self.tab_log_layout)?;

        /* Layout tab layout */
        FlexboxLayout::builder()
            .parent(&self.tab_container)
            .child(self.layout_view.view())
            .child_margin(Rect {
                start: PT(4.0),
                end: PT(16.0),
                top: PT(6.0),
                bottom: PT(40.0),
            })
            .build(&self.tab_layouts_layout)?;

        /* Main window */
        FlexboxLayout::builder()
            .parent(&self.window)
            .flex_direction(FlexDirection::Column)
            /* Tabs */
            .child(&self.tab_container)
            .child_flex_grow(1.0)
            /* Test label */
            .child(&self.key_event_label)
            .child_size(Size {
                width: D::Auto,
                height: D::Points(24.0),
            })
            /* Test editor */
            .child(self.test_editor.editor())
            .child_size(Size {
                width: D::Auto,
                height: D::Points(40.0),
            })
            .build(&self.layout)
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        self.main_menu.handle_event(app, evt, handle);
        self.tray.handle_event(app, evt, handle);
        self.test_editor.handle_event(evt);
        match evt {
            Event::OnWindowClose => {
                if &handle == &self.window.handle {
                    app.on_window_close();
                }
            }
            _ => {}
        }
    }

    pub(crate) fn handle(&self) -> ControlHandle {
        self.window.handle
    }

    pub(crate) fn hwnd(&self) -> HWND {
        hwnd(self.window.handle)
    }

    pub(crate) fn update_ui(
        &self,
        is_auto_switch_layout_enabled: bool,
        is_logging_enabled: bool,
        current_layout: &KeyTransformLayout,
    ) {
        self.main_menu.update_ui(
            is_auto_switch_layout_enabled,
            is_logging_enabled,
            current_layout,
        );
        self.tray.update_ui(current_layout);
    }

    pub(crate) fn apply_settings(&self, settings: &MainWindowSettings) {
        if let Some(position) = settings.position {
            self.window.set_position(position.0, position.1);
        }
        if let Some(size) = settings.size {
            ui::utils::set_window_size(&self.window, size);
        }
        if let Some(page) = settings.selected_page {
            self.tab_container.set_selected_tab(page);
        }
        self.log_view.apply_settings(settings);
    }

    pub(crate) fn update_settings(&self, settings: &mut MainWindowSettings) {
        settings.position = Some(self.window.position());
        settings.size = Some(ui::utils::get_window_size(&self.window));
        settings.selected_page = Some(self.tab_container.selected_tab());
        self.log_view.update_settings(settings);
    }

    pub(crate) fn set_layouts(&self, layouts: &KeyTransformLayouts) {
        self.main_menu.build_layouts_menu(layouts);
        self.tray.build_layout_menu(layouts);
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

    pub(crate) fn clear_log(&self) {
        self.log_view.clear()
    }

    pub(crate) fn on_layout_changed(&self, layout: Option<&KeyTransformLayout>) {
        self.layout_view.update_ui(layout);
    }

    pub(crate) fn on_key_hook_notify(&self, event: &KeyEvent) {
        self.log_view.append(event);
        self.key_event_label
            .set_text(event.as_trigger().to_string().as_str());
    }
}
