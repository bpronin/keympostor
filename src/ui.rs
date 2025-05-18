#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod ui_log_view;
mod ui_main;
mod ui_main_menu;
mod ui_profile_view;
mod ui_tray;
mod ui_util;

pub(crate) fn run_app() {
    ui_main::run();
}
