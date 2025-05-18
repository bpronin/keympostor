use crate::{rs, ui_panic, ui_warn};
use super::*;
use crate::keyboard::key_event::KeyEvent;
use crate::keyboard::key_hook::KeyboardHandler;
use crate::keyboard::transform_rules::KeyTransformProfile;
use crate::res::RESOURCE_STRINGS;
use crate::settings::AppSettings;
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use ui_log_view::LogView;
use ui_main_menu::MainMenu;
use ui_profile_view::ProfileView;
use ui_tray::Tray;
use crate::ui::ui_util::default_font;
use crate::util::default_profile_path;

#[derive(Default)]
pub(crate) struct App {
    keyboard_handler: KeyboardHandler,
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    tab_log_layout: nwg::FlexboxLayout,
    tab_profiles_layout: nwg::FlexboxLayout,
    text_editor: nwg::TextInput,
    tab_container: nwg::TabsContainer,
    tab_log: nwg::Tab,
    tab_profile: nwg::Tab,
    log_view: LogView,
    profile_view: ProfileView,
    main_menu: MainMenu,
    tray: Tray,
}

impl App {
    fn read_settings(&self) {
        let settings = AppSettings::load().unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });

        self.keyboard_handler
            .set_enabled(settings.key_processing_enabled);
        self.keyboard_handler
            .set_silent(settings.silent_key_processing);
    }

    fn write_settings(&self) {
        let mut settings = AppSettings::load().unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });

        settings.key_processing_enabled = self.keyboard_handler.is_enabled();
        settings.silent_key_processing = self.keyboard_handler.is_silent();

        settings.save().unwrap_or_else(|e| {
            ui_warn!("{}", e);
        });
    }

    fn read_profile(&self, path: &str) {
        let profile = KeyTransformProfile::load(path).unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });
        self.profile_view.update_ui(&profile);
        self.keyboard_handler.set_profile(profile);
    }

    fn update_controls(&self) {
        self.main_menu.update_ui(
            self.keyboard_handler.is_enabled(),
            self.keyboard_handler.is_silent(),
        );

        self.tray.update_ui(self.keyboard_handler.is_enabled());
    }

    pub(crate) fn run(&self) {
        self.read_settings();
        self.read_profile(&default_profile_path());

        self.update_controls();
        self.log_view.init();
        self.log_view
            .update_log_enabled(!self.keyboard_handler.is_silent());

        nwg::dispatch_thread_events();
    }

    fn on_window_close(&self) {
        // self.keyboard_handler.set_silent(true);
        #[cfg(feature = "dev")]
        self.on_app_exit();
    }

    pub(crate) fn on_toggle_processing_enabled(&self) {
        self.keyboard_handler
            .set_enabled(!self.keyboard_handler.is_enabled());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_toggle_logging_enabled(&self) {
        self.keyboard_handler
            .set_silent(!self.keyboard_handler.is_silent());

        self.log_view
            .update_log_enabled(!self.keyboard_handler.is_silent());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_app_exit(&self) {
        nwg::stop_thread_dispatch();
    }

    pub(crate) fn on_open_window(&self) {
        self.window.set_visible(true);
        // self.keyboard_handler.set_silent(false);
    }

    pub(crate) fn on_toggle_window_visibility(&self) {
        self.window.set_visible(!self.window.visible());
    }

    pub(crate) fn on_load_profile(&self) {
        let mut dialog = nwg::FileDialog::default();

        nwg::FileDialog::builder()
            .title(rs!(load_profile))
            .filters(rs!(load_profile_filter))
            .action(nwg::FileDialogAction::Open)
            .build(&mut dialog)
            .unwrap();

        if dialog.run(Some(self.window.handle)) {
            let path = dialog.get_selected_item().unwrap();
            self.read_profile(path.to_str().unwrap());
        }
    }

    pub(crate) fn on_log_view_clear(&self) {
        self.log_view.clear();
    }
}

struct AppUi {
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
            .flags(window_flags)
            .title(rs!(app_title))
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
            .text(rs!(log))
            .parent(&app.tab_container)
            .build(&mut app.tab_log)?;

        nwg::Tab::builder()
            .text(rs!(profile))
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
            .set_callback(Some(Box::new(kbd_handler_callback)));

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

pub(crate) fn run() {
    nwg::init().expect("Failed to init Native Windows GUI.");
    App::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}
