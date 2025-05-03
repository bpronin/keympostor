use super::*;
use crate::key_code::KeyCode;
use crate::key_hook::{KeyboardEvent, KeyboardHandler};
use crate::profile::Profile;
use crate::res::{Resources, RESOURCE_STRINGS};
use crate::res_ids::{IDI_ICON_GAME_LOCK_OFF, IDI_ICON_GAME_LOCK_ON};
use crate::settings::AppSettings;
use crate::transform::KeyTransformMap;
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

thread_local! {
    static APP: RefCell<AppUi> = RefCell::new(
        AppControl::build_ui(Default::default()).expect("Failed to build application UI")
    )
}
#[derive(Default)]
struct AppControl {
    keyboard_handler: KeyboardHandler,
    resources: Resources,
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    text_editor: nwg::TextInput,
    log_view: nwg::TextBox, //todo set log max capacity
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

        settings.save().unwrap_or_else(|e| {
            ui_warn!("{}", e);
        });
    }

    fn read_profile(&self) {
        let profile = Profile::load().unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });
        let rules = KeyTransformMap::from_rules(profile.transform_rules).unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });

        self.keyboard_handler.set_rules(rules)
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
            self.log_view
                .appendln(&format!("--- {}", &Profile::file_path()));
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
        self.keyboard_handler.set_enabled(false);
        nwg::stop_thread_dispatch();
    }

    fn on_window_close(&self) {
        #[cfg(feature = "dev")]
        self.on_app_exit();
    }

    fn on_open_window(&self) {
        self.window.set_visible(true);
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

    fn on_log_view_update(&self, event: &KeyboardEvent) {
        let action = event.action;
        let scancode = action.key.scancode.unwrap();
        let virtual_key = action.key.virtual_key.unwrap();
        let line = &format!(
            "{}{}{} T: {} | {:20} [{}] | {:20} [{}] | {}",
            if event.is_processable() { "!" } else { " " },
            if event.is_injected() { ">" } else { " " },
            if event.is_private() { "X" } else { " " },
            event.time(),
            scancode.name(),
            scancode,
            virtual_key.name(),
            virtual_key,
            action.transition
        );

        self.log_view.appendln(line);
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

        nwg::TextBox::builder()
            .parent(&app.window)
            .readonly(true)
            .font(Some(&Self::mono_font(15)))
            .build(&mut app.log_view)?;

        // Wrap-up

        let ui = AppUi {
            inner: Rc::new(app),
            default_handler: Default::default(),
        };

        // Events

        let evt_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |evt, _evt_data, handle| {
            if let Some(evt_ui) = evt_ui.upgrade() {
                // if &handle == &evt_ui.window {
                //    println!("{:?}", evt);
                // }
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
                        if &handle == &evt_ui.main_menu.clear_log_item {
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

        const DEFAULT_PADDING: D = D::Points(4.0);
        const DEFAULT_MARGIN: D = D::Points(4.0);

        const PADDING: Rect<D> = Rect {
            start: DEFAULT_PADDING,
            end: DEFAULT_PADDING,
            top: DEFAULT_PADDING,
            bottom: DEFAULT_PADDING,
        };

        const MARGIN: Rect<D> = Rect {
            start: DEFAULT_MARGIN,
            end: DEFAULT_MARGIN,
            top: DEFAULT_MARGIN,
            bottom: DEFAULT_MARGIN,
        };

        nwg::FlexboxLayout::builder()
            .parent(&ui.window)
            .flex_direction(FlexDirection::Column)
            .padding(PADDING)
            //Log view
            .child(&ui.log_view)
            .child_margin(MARGIN)
            .child_flex_grow(2.0)
            .child_size(Size {
                width: D::Auto,
                height: D::Auto,
            })
            //Text editor
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

// todo: try to get rid of it
fn on_key_event(event: &KeyboardEvent) {
    APP.with_borrow(|app| app.on_log_view_update(event))
}

pub fn run_app() {
    APP.with_borrow(|app| app.run());
}
