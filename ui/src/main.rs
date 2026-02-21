#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")] /* hides the console window */
use crate::app::App;
use crate::ui::app_ui::AppUI;
use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use log::LevelFilter::{Trace, Warn};
use std::error::Error;
use std::fs::File;
use std::io::stdout;
use std::thread;

mod app;
mod indicator;
mod kb_watch;
mod layout;
mod profile;
mod settings;
mod ui;
mod util;
mod win_watch;

fn main() {
    log_panics::init();
    setup_logger().expect("Failed to initialize logger.");

    let app = App::default();
    let ui = AppUI::build(app);
    ui.run();
}

fn setup_logger() -> Result<(), Box<dyn Error>> {
    let colors = ColoredLevelConfig::new()
        .info(Color::Blue)
        .debug(Color::Green);

    let stdout_config = Dispatch::new()
        .level(Trace)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} {:<5} [{}] {:<32} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                colors.color(record.level()),
                thread::current().name().unwrap_or("noname"),
                record.target(),
                message
            ))
        })
        .filter(|metadata| {
            metadata.level() <= Warn || metadata.target().starts_with("keympostor::")
        })
        .chain(stdout());

    let file_config = Dispatch::new()
        .level(Warn)
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

    /*
        error!("Error message");
        info!("Info message");
        debug!("Debug message");
        warn!("Warn message");
        trace!("Trace message");
    */

    Ok(())
}
