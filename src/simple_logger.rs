use crate::ios::ffi_for_sanity_tests::SENDER;
use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

pub struct SimpleLogger;

impl SimpleLogger {
    fn log_to_app(str: &str) {
        unsafe {
            if let Some(s) = &SENDER {
                s.send(str.to_owned()).expect("Couldn't send");
            } else {
                println!("No SENDER!");
            }
        }
    }
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
            SimpleLogger::log_to_app("test1");
        }
    }

    fn flush(&self) {}
}
