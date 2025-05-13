use super::*;
use crate::res::{Resources, RESOURCE_STRINGS};
use crate::res_ids::{IDI_ICON_GAME_LOCK_OFF, IDI_ICON_GAME_LOCK_ON};
use crate::settings::AppSettings;
use keyboard::key_event::KeyEvent;
use keyboard::key_hook::KeyboardHandler;
use keyboard::transform_rules::KeyTransformProfile;
use log::warn;
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

const MAX_LOG_LINES: usize = 256;

#[derive(Default)]
struct AppControl {
    keyboard_handler: KeyboardHandler,
    resources: Resources,
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    tab_log_layout: nwg::FlexboxLayout,
    tab_profiles_layout: nwg::FlexboxLayout,
    text_editor: nwg::TextInput,
    tab_container: nwg::TabsContainer,
    tab_log: nwg::Tab,
    tab_profile: nwg::Tab,
    log_view: nwg::TextBox,
    profile_view: nwg::TextBox,
    main_menu: MainMenu,
    tray: nwg::TrayNotification,
    tray_menu: TrayMenu,
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

#[derive(Default)]
struct TrayMenu {
    menu: nwg::Menu,
    toggle_processing_enabled_item: nwg::MenuItem,
    open_app_item: nwg::MenuItem,
    exit_app_item: nwg::MenuItem,
    separator: nwg::MenuSeparator,
}

impl AppControl {
    fn default_font(size: u32) -> nwg::Font {
        let mut font = nwg::Font::default();
        nwg::Font::builder()
            .family("Segoe UI")
            .size(size)
            .build(&mut font)
            .expect("Failed to build font");
        font
    }

    fn mono_font(size: u32) -> nwg::Font {
        let mut font = nwg::Font::default();
        nwg::Font::builder()
            .family("Consolas")
            .size(size)
            .build(&mut font)
            .expect("Failed to build font");
        font
    }

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

    fn read_profile(&self) {
        let profile = KeyTransformProfile::load(&profile_path()).unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });
        self.update_controls_profile_changed(&profile);
        self.keyboard_handler.set_profile(profile);
    }

    pub fn get_icon(&self, icon_id: usize) -> nwg::Icon {
        let mut icon = nwg::Icon::default();
        nwg::Icon::builder()
            .source_embed(Some(&self.resources.embedded))
            .source_embed_id(icon_id)
            .strict(true)
            .size(Some((16, 16)))
            .build(&mut icon)
            .unwrap_or_else(|e| {
                ui_panic!("{}", e);
            });
        icon
    }

    pub fn run(&self) {
        // let callback = |event: &KeyboardEvent| {self.on_log_view_update(event)};
        // let boxed_callback = Box::new(callback);

        let boxed_callback = Box::new(on_key_event);
        self.keyboard_handler.set_callback(Some(boxed_callback));

        self.read_settings();
        self.read_profile();
        self.update_controls();
        self.update_controls_logging_enabled();

        #[cfg(feature = "dev")]
        {
            self.log_view.appendln("--- Debug UI");
            self.log_view.appendln(&format!("--- {}", &profile_path()));
        }

        nwg::dispatch_thread_events();
    }

    fn update_controls(&self) {
        self.main_menu
            .toggle_processing_enabled_item
            .set_checked(self.keyboard_handler.is_enabled());

        self.main_menu
            .toggle_logging_enabled_item
            .set_checked(!self.keyboard_handler.is_silent());

        self.tray_menu
            .toggle_processing_enabled_item
            .set_checked(self.keyboard_handler.is_enabled());

        if self.keyboard_handler.is_enabled() {
            self.tray.set_icon(&self.get_icon(IDI_ICON_GAME_LOCK_ON));
        } else {
            self.tray.set_icon(&self.get_icon(IDI_ICON_GAME_LOCK_OFF));
        }
    }

    fn update_controls_logging_enabled(&self) {
        if self.keyboard_handler.is_silent() {
            self.log_view.appendln(rs!(_logging_disabled_));
        } else {
            self.log_view.appendln(rs!(_logging_enabled_));
        }
    }

    fn update_controls_profile_changed(&self, profile: &KeyTransformProfile) {
        let s = profile.to_string();
        self.profile_view.set_text(&s);
    }

    fn on_toggle_processing_enabled(&self) {
        self.keyboard_handler
            .set_enabled(!self.keyboard_handler.is_enabled());
        self.update_controls();
        self.write_settings();
    }

    fn on_toggle_logging_enabled(&self) {
        self.keyboard_handler
            .set_silent(!self.keyboard_handler.is_silent());

        self.update_controls_logging_enabled();
        self.update_controls();
        self.write_settings();
    }

    fn on_app_exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn on_window_close(&self) {
        // self.keyboard_handler.set_silent(true);
        #[cfg(feature = "dev")]
        self.on_app_exit();
    }

    fn on_open_window(&self) {
        self.window.set_visible(true);
        // self.keyboard_handler.set_silent(false);
    }

    fn on_toggle_window_visibility(&self) {
        self.window.set_visible(!self.window.visible());
    }

    fn on_tray_menu_show(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.menu.popup(x, y);
    }

    fn on_log_view_clear(&self) {
        self.log_view.clear();
    }

    fn on_load_profile(&self) {
        warn!("load profile not implemented");
    }

    fn trim_log_text(&self) {
        let text = self.log_view.text();

        let skip_count = text.lines().count().saturating_sub(MAX_LOG_LINES);
        let trimmed_text = text
            .lines()
            .skip(skip_count)
            .fold(String::new(), |mut acc, line| {
                acc.push_str(line);
                acc.push_str("\r\n");
                acc
            });

        self.log_view.set_text(&trimmed_text);
    }

    fn on_log_view_update(&self, event: &KeyEvent) {
        let action = event.action();
        let key = action.key;
        let scan_code = key.scan_code();
        let virtual_key = key.virtual_key();
        let line = format!(
            "{:1}{:1}{:1} T: {:9} | {:20}| {:22}| {:18} | {:1}",
            if event.rule.is_some() { "!" } else { "" },
            if event.is_injected() { ">" } else { "" },
            if event.is_private() { "<" } else { "" },
            event.time(),
            key,
            virtual_key,
            scan_code,
            action.transition
        );

        self.trim_log_text();
        self.log_view.appendln(&line);
    }
}

struct AppUi {
    inner: Rc<AppControl>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl NativeUi<AppUi> for AppControl {
    fn build_ui(mut app: AppControl) -> Result<AppUi, nwg::NwgError> {
        nwg::init().expect("Failed to init Native Windows GUI");
        nwg::Font::set_global_default(Self::default_font(17).into());

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
            .text(rs!(profile))
            .build(&mut app.main_menu.load_profile_item)?;

        nwg::MenuSeparator::builder()
            .parent(&app.main_menu.menu)
            .build(&mut app.main_menu.separator)?;

        nwg::MenuItem::builder()
            .parent(&app.main_menu.menu)
            .text(rs!(exit))
            .build(&mut app.main_menu.exit_app_item)?;

        // Tray icon

        nwg::TrayNotification::builder()
            .parent(&app.window)
            .icon(Some(&app.get_icon(IDI_ICON_GAME_LOCK_OFF)))
            .tip(Some(rs!(tray_tip)))
            .build(&mut app.tray)?;

        // Tray menu

        nwg::Menu::builder()
            .popup(true)
            .parent(&app.window)
            .build(&mut app.tray_menu.menu)?;

        nwg::MenuItem::builder()
            .parent(&app.tray_menu.menu)
            .text(rs!(enabled))
            .build(&mut app.tray_menu.toggle_processing_enabled_item)?;

        nwg::MenuItem::builder()
            .text(rs!(open))
            .parent(&app.tray_menu.menu)
            .build(&mut app.tray_menu.open_app_item)?;

        nwg::MenuSeparator::builder()
            .parent(&app.tray_menu.menu)
            .build(&mut app.tray_menu.separator)?;

        nwg::MenuItem::builder()
            .text(rs!(exit))
            .parent(&app.tray_menu.menu)
            .build(&mut app.tray_menu.exit_app_item)?;

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

        nwg::TextBox::builder()
            .parent(&app.tab_log)
            .readonly(true)
            .font(Some(&Self::mono_font(15)))
            .build(&mut app.log_view)?;

        nwg::TextBox::builder()
            .parent(&app.tab_profile)
            .readonly(true)
            .font(Some(&Self::mono_font(15)))
            .build(&mut app.profile_view)?;

        // Wrap-up

        let ui = AppUi {
            inner: Rc::new(app),
            default_handler: Default::default(),
        };

        // Events

        let evt_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |evt, _evt_data, handle| {
            if let Some(evt_ui) = evt_ui.upgrade() {
                match evt {
                    nwg::Event::OnWindowClose => {
                        if &handle == &evt_ui.window {
                            evt_ui.on_window_close();
                        }
                    }
                    nwg::Event::OnMousePress(nwg::MousePressEvent::MousePressLeftUp) => {
                        if &handle == &evt_ui.tray {
                            evt_ui.on_toggle_window_visibility();
                        }
                    }
                    nwg::Event::OnContextMenu => {
                        if &handle == &evt_ui.tray {
                            evt_ui.on_tray_menu_show();
                        }
                    }
                    nwg::Event::OnMenuItemSelected => {
                        if &handle == &evt_ui.main_menu.load_profile_item {
                            evt_ui.on_load_profile();
                        } else if &handle == &evt_ui.main_menu.clear_log_item {
                            evt_ui.on_log_view_clear();
                        } else if &handle == &evt_ui.main_menu.exit_app_item
                            || &handle == &evt_ui.tray_menu.exit_app_item
                        {
                            evt_ui.on_app_exit();
                        } else if &handle == &evt_ui.tray_menu.open_app_item {
                            evt_ui.on_open_window();
                        } else if &handle == &evt_ui.main_menu.toggle_logging_enabled_item {
                            evt_ui.on_toggle_logging_enabled();
                        } else if &handle == &evt_ui.main_menu.toggle_processing_enabled_item
                            || &handle == &evt_ui.tray_menu.toggle_processing_enabled_item
                        {
                            evt_ui.on_toggle_processing_enabled();
                        }
                    }
                    _ => {}
                }
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
            .child(&ui.log_view)
            .child_margin(TAB_MARGIN)
            .child_flex_grow(1.0)
            .build(&ui.tab_log_layout)?;

        /* Profile tab layout */
        nwg::FlexboxLayout::builder()
            .parent(&ui.tab_container)
            .padding(TAB_PADDING)
            .child(&ui.profile_view)
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

pub(crate) fn profile_path() -> String {
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
