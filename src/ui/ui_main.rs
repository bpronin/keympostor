use super::*;
use crate::keyboard::key_event::KeyEvent;
use crate::res::res_ids::{IDI_ICON_APP, IDS_APP_TITLE, IDS_LOG, IDS_PROFILE};
use crate::res::RES;
use crate::ui::ui_util::default_font;
use crate::{r_icon, rs};
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub(crate) struct AppUi {
    app: Rc<App>,
    event_handler: RefCell<Option<nwg::EventHandler>>,
}

impl AppUi {
    fn build(mut app: App) -> Result<Self, nwg::NwgError> {
        #[cfg(not(feature = "dev"))]
        let window_flags = nwg::WindowFlags::MAIN_WINDOW;

        #[cfg(feature = "dev")]
        let window_flags = nwg::WindowFlags::MAIN_WINDOW | nwg::WindowFlags::VISIBLE;

        nwg::Window::builder()
            .size((700, 300))
            .icon(Some(r_icon!(IDI_ICON_APP)))
            .flags(window_flags)
            .title(rs!(IDS_APP_TITLE))
            .build(&mut app.window)?;

        nwg::TextInput::builder()
            .parent(&app.window)
            .focus(true)
            .build(&mut app.text_editor)?;

        /* Tabs */

        nwg::TabsContainer::builder()
            .parent(&app.window)
            .build(&mut app.tab_container)?;

        nwg::Tab::builder()
            .text(rs!(IDS_LOG))
            .parent(&app.tab_container)
            .build(&mut app.tab_log)?;

        nwg::Tab::builder()
            .text(rs!(IDS_PROFILE))
            .parent(&app.tab_container)
            .build(&mut app.tab_profile)?;

        app.main_menu.build_ui(&mut app.window)?;
        app.tray.build_ui(&mut app.window)?;
        app.log_view.build_ui(&mut app.tab_log)?;
        app.profile_view.build_ui(&mut app.tab_profile)?;

        /* Wrap-up */

        Ok(Self {
            app: Rc::new(app),
            event_handler: Default::default(),
        })
    }

    fn setup_event_handlers(&self) {
        /* Components callbacks */

        let app_rc = Rc::downgrade(&self.app);
        let kbd_handler_callback = move |event: &KeyEvent| {
            if let Some(app) = app_rc.upgrade() {
                app.log_view.update_ui(event);
            }
        };
        self.app
            .keyboard_handler
            .set_listener(Some(Box::new(kbd_handler_callback)));

        /* Windows events */

        let app_rc = Rc::downgrade(&self.app);
        let event_handler = move |evt, _evt_data, handle| {
            if let Some(app) = app_rc.upgrade() {
                app.tray.handle_event(&app, evt, handle);
                app.main_menu.handle_event(&app, evt, handle);

                match evt {
                    nwg::Event::OnWindowClose => {
                        if &handle == &app.window {
                            app.on_window_close();
                        }
                    }
                    _ => {}
                };
            }
        };

        *self.event_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &self.window.handle,
            event_handler,
        ));
    }

    fn layout(&self) -> Result<(), nwg::NwgError> {
        use nwg::stretch::{
            geometry::{Rect, Size},
            style::{Dimension as D, FlexDirection},
        };

        const PADDING: Rect<D> = Rect {
            start: D::Points(4.0),
            end: D::Points(4.0),
            top: D::Points(4.0),
            bottom: D::Points(4.0),
        };

        const TAB_PADDING: Rect<D> = Rect {
            start: D::Points(0.0),
            end: D::Points(8.0),
            top: D::Points(0.0),
            bottom: D::Points(4.0),
        };

        const MARGIN: Rect<D> = Rect {
            start: D::Points(4.0),
            end: D::Points(4.0),
            top: D::Points(4.0),
            bottom: D::Points(4.0),
        };

        const TAB_MARGIN: Rect<D> = Rect {
            start: D::Points(4.0),
            end: D::Points(4.0),
            top: D::Points(4.0),
            bottom: D::Points(18.0),
        };

        /* Log tab layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.tab_container)
            .padding(TAB_PADDING)
            .child(self.log_view.view())
            .child_margin(TAB_MARGIN)
            .child_flex_grow(1.0)
            .build(&self.tab_log_layout)?;

        /* Profile tab layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.tab_container)
            .padding(TAB_PADDING)
            .child(self.profile_view.view())
            .child_margin(TAB_MARGIN)
            .child_flex_grow(1.0)
            .build(&self.tab_profiles_layout)?;

        /* Main window layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.window)
            .flex_direction(FlexDirection::Column)
            .padding(PADDING)
            /* Tabs */
            .child(&self.tab_container)
            .child_margin(MARGIN)
            .child_flex_grow(1.0)
            /* Test editor */
            .child(&self.text_editor)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Auto,
                height: D::Points(32.0),
            })
            .build(&self.layout)
    }
}

impl NativeUi<AppUi> for App {
    fn build_ui(app: App) -> Result<AppUi, nwg::NwgError> {
        nwg::Font::set_global_default(default_font(17).into());

        let ui = AppUi::build(app)?;
        ui.setup_event_handlers();
        ui.layout()?;

        Ok(ui)
    }
}

impl Drop for AppUi {
    fn drop(&mut self) {
        let handler = self.event_handler.borrow();
        if handler.is_some() {
            nwg::unbind_event_handler(handler.as_ref().unwrap());
        }
    }
}

impl Deref for AppUi {
    type Target = App;

    fn deref(&self) -> &App {
        &self.app
    }
}
