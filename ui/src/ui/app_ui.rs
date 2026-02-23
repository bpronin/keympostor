use crate::app::App;
use crate::rs;
use crate::ui::res::RESOURCES;
use crate::ui::res_ids::IDS_APP_ALREADY_RUNNING;
use crate::ui::style::display_font;
use crate::ui::utils::show_warn_message;
use crate::util::is_app_running;
use native_windows_gui as nwg;
use std::rc::Rc;

#[derive(Default)]
pub struct AppUI {
    app: Rc<App>,
}

impl AppUI {
    pub(crate) fn build(mut app: App) -> Self {
        nwg::init().expect("Failed to init Native Windows GUI.");
        nwg::Font::set_global_default(Some(display_font(17)));

        app.window.build().expect("Failed to build main window.");

        Self { app: Rc::new(app) }
    }

    pub(crate) fn run(&self) {
        #[cfg(not(feature = "debug"))]
        if is_app_running() {
            show_warn_message(rs!(IDS_APP_ALREADY_RUNNING));
            return;
        }
        self.setup_event_handlers();
        nwg::dispatch_thread_events();
    }

    fn setup_event_handlers(&self) {
        let app_rc = Rc::downgrade(&self.app);

        nwg::full_bind_event_handler(&self.app.window.handle(), move |evt, _evt_data, handle| {
            if let Some(app) = app_rc.upgrade() {
                app.handle_event(evt, handle);
            }
        });

        let app_rc = Rc::downgrade(&self.app);
        nwg::bind_raw_event_handler(
            &self.app.window.handle(),
            0x10000,
            move |_hwnd, msg, _w_param, l_param| {
                if let Some(app) = app_rc.upgrade() {
                    app.handle_raw_event(msg, l_param);
                }
                None
            },
        )
        .expect("Failed to bind raw event handler");
    }
}
