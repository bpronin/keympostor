use keympostor::win_watcher::WindowWatcher;
use log::LevelFilter;
use regex::Regex;
use simple_logger::SimpleLogger;
use std::str::FromStr;
use std::sync::Arc;

fn setup_logger() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");
}

fn setup_exit_handler(watcher: &Arc<WindowWatcher>) {
    let w = Arc::clone(&watcher);
    ctrlc::set_handler(move || w.stop()).unwrap();
}

fn main() {
    setup_logger();
    let watcher = Arc::new(WindowWatcher::new());
    setup_exit_handler(&watcher);

    println!("Running... (press Ctrl+C to stop)");

    let regex = Regex::from_str("Chrome").unwrap();
    watcher.start(regex, |hwnd, x| {
        println!("App [{:?}] is active: {}", hwnd, x);
    });
}
