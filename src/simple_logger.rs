use crate::ios::ffi_for_sanity_tests::{LOG_SENDER, CoreLogMessageThreadSafe, CoreLogLevel};
// use log::{Level, Metadata, Record};
// use log::{LevelFilter, SetLoggerError};
use chrono::Utc;
use log::*;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

pub struct SimpleLogger;

impl SimpleLogger {
    // fn log_to_app(str: &str) {
    //     unsafe {
    //         if let Some(s) = &SENDER {
    //             s.send(str.to_owned()).expect("Couldn't send");
    //         } else {
    //             println!("No SENDER!");
    //         }
    //     }
    // }
    fn log_message_to_app(log_message: CoreLogMessageThreadSafe){
        unsafe {
            if let Some(s) = &LOG_SENDER {
                s.send(log_message).expect("Couldn't send");
            } else {
                warn!("No SENDER!");
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
            let arg_string = format!("{}", record.args());
            let lvl = match record.level(){
                Level::Debug => CoreLogLevel::Debug,
                Level::Error => CoreLogLevel::Error,
                Level::Info => CoreLogLevel::Info,
                Level::Warn => CoreLogLevel::Warn,
                Level::Trace => CoreLogLevel::Trace,
            };


            let lmts = CoreLogMessageThreadSafe{
                level: lvl,
                text: arg_string,
                time: Utc::now().timestamp(),
            };

            SimpleLogger::log_message_to_app(lmts);
        }
    }

    fn flush(&self) {}
}
