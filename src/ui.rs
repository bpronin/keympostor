use super::*;
use crate::res::RESOURCE_STRINGS;
use crate::settings::AppSettings;
use crate::ui_log_view::LogView;
use crate::ui_main_menu::MainMenu;
use crate::ui_profile_view::ProfileView;
use crate::ui_tray::Tray;
use crate::util::default_font;
use keyboard::key_event::KeyEvent;
use keyboard::key_hook::KeyboardHandler;
use keyboard::transform_rules::KeyTransformProfile;
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use crate::util;

#[derive(Default)]
pub(crate) struct AppControl {
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

impl AppControl {
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
        self.read_profile(&util::default_profile_path());

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

    fn on_log_view_update(&self, event: &KeyEvent) {
        self.log_view.update_ui(event);
    }
}

struct AppUi {
    app: Rc<AppControl>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl NativeUi<AppUi> for AppControl {
    fn build_ui(mut app: AppControl) -> Result<AppUi, nwg::NwgError> {
        nwg::init().expect("Failed to init Native Windows GUI.");
        nwg::Font::set_global_default(default_font(17).into());

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

        let ui = AppUi {
            app: Rc::new(app),
            default_handler: Default::default(),
        };

        /* Components callbacks */

        let app_rc = Rc::downgrade(&ui.app);
        let kbd_handler_callback = move |event: &KeyEvent| {
            if let Some(app) = app_rc.upgrade() {
                app.on_log_view_update(event)
            }
        };
        ui.app.keyboard_handler.set_callback(Some(Box::new(kbd_handler_callback)));

        /* Windows events */
        
        let app_rc = Rc::downgrade(&ui.app);
        let default_handler = move |evt, _evt_data, handle| {
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

        *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.window.handle,
            default_handler,
        ));

        /* Layout */

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
            .parent(&ui.tab_container)
            .padding(TAB_PADDING)
            .child(ui.log_view.view())
            .child_margin(TAB_MARGIN)
            .child_flex_grow(1.0)
            .build(&ui.tab_log_layout)?;

        /* Profile tab layout */
        nwg::FlexboxLayout::builder()
            .parent(&ui.tab_container)
            .padding(TAB_PADDING)
            .child(ui.profile_view.view())
            .child_margin(TAB_MARGIN)
            .child_flex_grow(1.0)
            .build(&ui.tab_profiles_layout)?;

        /* Main window layout */
        nwg::FlexboxLayout::builder()
            .parent(&ui.window)
            .flex_direction(FlexDirection::Column)
            .padding(PADDING)
            /* Tabs */
            .child(&ui.tab_container)
            .child_margin(MARGIN)
            .child_flex_grow(1.0)
            /* Test editor */
            .child(&ui.text_editor)
            .child_margin(MARGIN)
            .child_size(Size {
                width: D::Auto,
                height: D::Points(32.0),
            })
            .build(&ui.layout)?;

        Ok(ui)
    }
}

impl Drop for AppUi {
    fn drop(&mut self) {
        let handler = self.default_handler.borrow();
        if handler.is_some() {
            nwg::unbind_event_handler(handler.as_ref().unwrap());
        }
    }
}

impl Deref for AppUi {
    type Target = AppControl;

    fn deref(&self) -> &AppControl {
        &self.app
    }
}

pub fn run_app() {
    AppControl::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}
