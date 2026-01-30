#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")] /* hides the console window */
use crate::ui::App;
use log::LevelFilter;
use native_windows_gui::NativeUi;
use simple_logger::SimpleLogger;

pub mod indicator;
mod kb_watch;
mod layout;
mod profile;
mod res;
mod settings;
mod ui;
mod util;
mod win_watch;

fn main() {
    SimpleLogger::new()
        .with_module_level("keympostor", LevelFilter::Trace)
        .with_level(LevelFilter::Warn)
        .init()
        .expect("Failed to initialize logger.");

    App::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}
