#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use crate::ui::App;
use log::LevelFilter;
use native_windows_gui::NativeUi;
use simple_logger::SimpleLogger;

mod profile;
mod res;
mod settings;
mod ui;
mod win_watch;
mod kblight;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");

    App::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}
