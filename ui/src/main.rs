#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use ui::run_app;
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod ui;
mod res;
mod settings;
mod utils;
pub mod win_watch;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");

    run_app();
}
