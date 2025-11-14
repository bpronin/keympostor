use super::*;
use crate::res::res_ids::{IDI_ICON_APP, IDS_APP_TITLE, IDS_LAYOUT, IDS_LOG};
use crate::res::RESOURCES;
use crate::ui::style::{
    display_font, BIG_MONO_FONT, INFO_LABEL_FONT, MARGIN, MARGIN_2, PADDING, TAB_MARGIN,
    TAB_PADDING,
};
use crate::{r_icon, rs};
use keympostor::keyboard::event::KeyEvent;
use keympostor::keyboard::hook::WM_KEY_HOOK_NOTIFY;
use log::error;
use native_windows_gui as nwg;
use native_windows_gui::stretch::geometry::Rect;
use native_windows_gui::stretch::style::Dimension;
use nwg::stretch::style::Dimension::Points as PT;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct MainWindow {
    app: Rc<App>,
    event_handler: RefCell<Option<nwg::EventHandler>>,
    raw_event_handler: RefCell<Option<nwg::RawEventHandler>>,
}

impl MainWindow {
    pub(crate) fn build(mut app: App) -> Result<Self, nwg::NwgError> {

        nwg::Window::builder()
            .size((700, 300))
            .icon(Some(r_icon!(IDI_ICON_APP)))
            .flags(nwg::WindowFlags::MAIN_WINDOW)
            .title(rs!(IDS_APP_TITLE))
            .build(&mut app.window)?;

        nwg::Label::builder()
            .parent(&app.window)
            .text("*")
            .font(Some(&INFO_LABEL_FONT))
            .build(&mut app.key_event_label)?;

        app.test_editor.build(&mut app.window)?;

        /* Tabs */

        nwg::TabsContainer::builder()
            .parent(&app.window)
            .build(&mut app.tab_container)?;

        nwg::Tab::builder()
            .text(rs!(IDS_LOG))
            .parent(&app.tab_container)
            .build(&mut app.tab_log)?;

        nwg::Tab::builder()
            .text(rs!(IDS_LAYOUT))
            .parent(&app.tab_container)
            .build(&mut app.tab_layouts)?;

        app.main_menu.build(&mut app.window)?;
        app.tray.build(&mut app.window)?;
        app.log_view.build(&mut app.tab_log)?;
        app.layout_view.build(&mut app.tab_layouts)?;

        let this = Self {
            app: Rc::new(app),
            event_handler: Default::default(),
            raw_event_handler: Default::default(),
        };
        this.layout()?;

        Ok(this)
    }

    fn layout(&self) -> Result<(), nwg::NwgError> {
        use nwg::stretch::{geometry::Size, style::FlexDirection};

        /* Log tab layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.app.tab_container)
            // .padding(TAB_PADDING)
            .child(self.app.log_view.view())
            .child_margin(Rect {
                start: PT(4.0),
                end: PT(16.0),
                top: PT(6.0),
                bottom: PT(40.0),
            })
            .build(&self.app.tab_log_layout)?;

        /* Layout tab layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.app.tab_container)
            // .padding(TAB_PADDING)
            .child(self.app.layout_view.view())
            .child_margin(Rect {
                start: PT(4.0),
                end: PT(16.0),
                top: PT(6.0),
                bottom: PT(40.0),
            })
            .build(&self.app.tab_layouts_layout)?;

        /* Main window layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.app.window)
            .flex_direction(FlexDirection::Column)
            // .padding(PADDING)
            /* Tabs */
            .child(&self.app.tab_container)
            // .child_margin(MARGIN)
            .child_flex_grow(1.0)
            /* Test label */
            .child(&self.app.key_event_label)
            // .child_margin(MARGIN_2)
            .child_size(Size {
                width: Dimension::Auto,
                height: Dimension::Points(24.0),
            })
            /* Test editor */
            .child(self.app.test_editor.editor())
            // .child_margin(MARGIN_2)
            .child_size(Size {
                width: Dimension::Auto,
                height: Dimension::Points(40.0),
            })
            .build(&self.app.layout)
    }

    fn setup_event_handlers(&self) {
        let app_rc = Rc::downgrade(&self.app);
        self.event_handler
            .replace(Some(nwg::full_bind_event_handler(
                &self.app.window.handle,
                move |evt, _evt_data, handle| {
                    // debug!("NWG: {:?} {:?} {:?}", evt, _evt_data, handle);
                    if let Some(app) = app_rc.upgrade() {
                        app.handle_event(evt, handle);
                    }
                },
            )));

        let app_rc = Rc::downgrade(&self.app);
        self.raw_event_handler.replace(Some(
            nwg::bind_raw_event_handler(
                &self.app.window.handle,
                0x10000,
                move |_hwnd, msg, _w_param, l_param| {
                    // debug!("NWG RAW: {:?} {:?} {:?} {:?}", _hwnd, msg, _w_param, l_param);
                    if let Some(app) = app_rc.upgrade() {
                        app.handle_raw_event(msg, l_param);
                    }
                    None
                },
            )
            .expect("Failed to bind raw event handler"),
        ));
    }

    pub(crate) fn run(&self) {
        self.setup_event_handlers();
        self.app.run()
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        if let Some(handler) = self.event_handler.borrow().as_ref() {
            nwg::unbind_event_handler(handler);
        }
        if let Some(handler) = self.raw_event_handler.borrow().as_ref() {
            nwg::unbind_raw_event_handler(handler)
                .unwrap_or_else(|e| error!("Failed to unbind raw event handler: {}", e));
        }
    }
}
