use super::*;
use crate::res::RESOURCE_STRINGS;
use crate::settings::AppSettings;
use crate::ui_log_view::LogView;
use crate::ui_profile_view::ProfileView;
use crate::ui_tray::Tray;
use crate::util::default_font;
use keyboard::key_event::KeyEvent;
use keyboard::key_hook::KeyboardHandler;
use keyboard::transform_rules::KeyTransformProfile;
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::RefCell;
use std::env;
use std::ops::Deref;
use std::rc::Rc;

thread_local! {
    static APP: RefCell<AppUi> = RefCell::new(
        AppControl::build_ui(Default::default()).expect("Failed to build application UI")
    )
}

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

#[derive(Default)]
struct MainMenu {
    menu: nwg::Menu,
    toggle_processing_enabled_item: nwg::MenuItem,
    toggle_logging_enabled_item: nwg::MenuItem,
    clear_log_item: nwg::MenuItem,
    load_profile_item: nwg::MenuItem,
    separator: nwg::MenuSeparator,
    exit_app_item: nwg::MenuItem,
}

impl MainMenu {
    pub(crate) fn update_ui(&self, is_processing_enabled: bool, is_silent: bool) {
        self.toggle_processing_enabled_item
            .set_checked(is_processing_enabled);

        self.toggle_logging_enabled_item.set_checked(!is_silent);
    }
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
        // let callback = |event: &KeyboardEvent| {self.on_log_view_update(event)};
        // let boxed_callback = Box::new(callback);

        let boxed_callback = Box::new(on_key_event);
        self.keyboard_handler.set_callback(Some(boxed_callback));

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

    fn on_load_profile(&self) {
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

    fn on_log_view_clear(&self) {
        self.log_view.clear();
    }

    fn on_log_view_update(&self, event: &KeyEvent) {
        self.log_view.update_ui(event);
    }
}

struct AppUi {
    inner: Rc<AppControl>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl NativeUi<AppUi> for AppControl {
    fn build_ui(mut app: AppControl) -> Result<AppUi, nwg::NwgError> {
        nwg::init().expect("Failed to init Native Windows GUI");
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

        // Main menu

        nwg::Menu::builder()
            .parent(&app.window)
            .text(rs!(file))
            .build(&mut app.main_menu.menu)?;

        nwg::MenuItem::builder()
            .parent(&app.main_menu.menu)
            .text(rs!(enabled))
            .build(&mut app.main_menu.toggle_processing_enabled_item)?;

        nwg::MenuSeparator::builder()
            .parent(&app.main_menu.menu)
            .build(&mut app.main_menu.separator)?;

        nwg::MenuItem::builder()
            .parent(&app.main_menu.menu)
            .text(rs!(logging_enabled))
            .build(&mut app.main_menu.toggle_logging_enabled_item)?;

        nwg::MenuItem::builder()
            .parent(&app.main_menu.menu)
            .text(rs!(clear_log))
            .build(&mut app.main_menu.clear_log_item)?;

        nwg::MenuItem::builder()
            .parent(&app.main_menu.menu)
            .text(rs!(load_profile))
            .build(&mut app.main_menu.load_profile_item)?;

        nwg::MenuSeparator::builder()
            .parent(&app.main_menu.menu)
            .build(&mut app.main_menu.separator)?;

        nwg::MenuItem::builder()
            .parent(&app.main_menu.menu)
            .text(rs!(exit))
            .build(&mut app.main_menu.exit_app_item)?;

        app.tray.build_ui(&app.window)?;

        // Main view

        nwg::TextInput::builder()
            .parent(&app.window)
            .focus(true)
            .build(&mut app.text_editor)?;

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

        app.log_view.build_ui(&app.tab_log)?;
        app.profile_view.build_ui(&app.tab_profile)?;

        // Wrap-up

        let ui = AppUi {
            inner: Rc::new(app),
            default_handler: Default::default(),
        };

        // Events

        let evt_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |evt, _evt_data, handle| {
            if let Some(evt_ui) = evt_ui.upgrade() {
                evt_ui.tray.handle_event(&evt_ui, evt, handle);

                match evt {
                    nwg::Event::OnWindowClose => {
                        if &handle == &evt_ui.window {
                            evt_ui.on_window_close();
                        }
                    }
                    nwg::Event::OnMenuItemSelected => {
                        if &handle == &evt_ui.main_menu.load_profile_item {
                            evt_ui.on_load_profile();
                        } else if &handle == &evt_ui.main_menu.clear_log_item {
                            evt_ui.on_log_view_clear();
                        } else if &handle == &evt_ui.main_menu.exit_app_item {
                            evt_ui.on_app_exit();
                        } else if &handle == &evt_ui.main_menu.toggle_logging_enabled_item {
                            evt_ui.on_toggle_logging_enabled();
                        } else if &handle == &evt_ui.main_menu.toggle_processing_enabled_item {
                            evt_ui.on_toggle_processing_enabled();
                        }
                    }
                    _ => {}
                };
            }
        };

        *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.window.handle,
            handle_events,
        ));

        // Layout

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
    // To make sure that everything is freed without issues, the default handler must be unbound.
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
        &self.inner
    }
}

pub(crate) fn default_profile_path() -> String {
    let mut args = env::args();
    args.next(); /* executable name */
    args.next().unwrap_or("profiles/default.toml".to_string())
}

// todo: try to get rid of it
fn on_key_event(event: &KeyEvent) {
    APP.with_borrow(|app| app.on_log_view_update(event));
}

pub fn run_app() {
    APP.with_borrow(|app| app.run());
}
