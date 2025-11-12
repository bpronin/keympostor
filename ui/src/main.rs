#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use log::LevelFilter;
use simple_logger::SimpleLogger;
use ui::run_app;

mod profile;
mod res;
mod settings;
mod ui;
mod utils;
mod win_watch;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");

    run_app();
}
