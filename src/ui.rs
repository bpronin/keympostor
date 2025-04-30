use super::*;
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use crate::res_ids::IDI_ICON_GAME_LOCK_OFF;

pub(crate) struct AppUi {
    inner: Rc<App>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl NativeUi<AppUi> for App {
    fn build_ui(mut app: App) -> Result<AppUi, nwg::NwgError> {
        nwg::init().expect("Failed to init Native Windows GUI");
        nwg::Font::set_global_default(default_font(17).into());

        let mut window_flags = nwg::WindowFlags::MAIN_WINDOW;
        
        #[cfg(feature = "dev")]
        {
            window_flags |= nwg::WindowFlags::VISIBLE;
        }

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
            .font(Some(&mono_font(15)))
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
    type Target = App;

    fn deref(&self) -> &App {
        &self.inner
    }
}

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
