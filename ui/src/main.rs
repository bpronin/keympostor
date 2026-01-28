#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")] /* hides the console window */

use crate::ui::App;
use log::LevelFilter;
use native_windows_gui::NativeUi;
use simple_logger::SimpleLogger;

mod kb_watch;
mod layout;
mod profile;
mod res;
mod settings;
mod ui;
mod win_watch;
pub mod indicator;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");

    App::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}
