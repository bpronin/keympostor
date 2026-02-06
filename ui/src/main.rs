#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")] /* hides the console window */
use std::error::Error;
use crate::ui::App;
use log::LevelFilter;
use native_windows_gui::NativeUi;
use std::fs::File;
use std::io::stdout;
use std::thread;
use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;

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
    log_panics::init();
    setup_logger().expect("Failed to initialize logger.");

    App::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}

fn setup_logger() -> Result<(), Box<dyn Error>> {
    let stdout_config = Dispatch::new()
        .level(LevelFilter::Debug)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} {:<5} [{}] {:<32} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                ColoredLevelConfig::new()
                    .error(Color::Red)
                    .warn(Color::Yellow)
                    .info(Color::Blue)
                    .debug(Color::Green)
                    .trace(Color::Magenta)
                    .color(record.level()),
                thread::current().name().unwrap_or("noname"),
                record.target(),
                message
            ))
        })
        .filter(|metadata| metadata.target().starts_with("keympostor::"))
        .chain(stdout());

    let file_config = Dispatch::new()
        .level(LevelFilter::Warn)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} {:<5} ({}) [{:<32}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                thread::current().name().unwrap_or("unknown"),
                record.target(),
                message
            ))
        })
        .chain(File::create("keympostor.log")?);

    Dispatch::new()
        .chain(stdout_config)
        .chain(file_config)
        .apply()?;

    Ok(())
}
