use crate::ios::ffi_for_sanity_tests::{CoreLogLevel, CoreLogMessageThreadSafe, LOG_SENDER};
// use log::{Level, Metadata, Record};
// use log::{LevelFilter, SetLoggerError};
use chrono::{Local, Utc};
use log::*;
use std::sync::Once;

static LOGGER: SimpleLogger = SimpleLogger;
static INIT: Once = Once::new();

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}

pub fn setup_with_level(level: LevelFilter){
    // Guaranteed to be executed only once (even if called multiple times).
    INIT.call_once(|| {
        log::set_logger(&LOGGER).map(|()| log::set_max_level(level)).expect("Logger initialization failed!");

    });

    // let resulting_level = log::max_level();
    // info!("Resulting level : {}", log::max_level());
    // println!("STATIC_MAX_LEVEL : {}", log::STATIC_MAX_LEVEL);
    // info!("Trace log level enabled: {}", log_enabled!(Level::Trace));
    // info!("Debug log level enabled: {}", log_enabled!(Level::Debug));
}

pub fn setup(){
    setup_with_level(LevelFilter::Trace);
}

pub struct SimpleLogger;

#[cfg(not(test))]
impl SimpleLogger {
    fn log_message_to_app(log_message: CoreLogMessageThreadSafe) {
        unsafe {
            if let Some(s) = &LOG_SENDER {
                s.send(log_message).expect("Couldn't send");
            } else {
                println!("No SENDER!");
            }
        }
    }
}

#[cfg(not(test))]
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // println!("metadata level : {}", metadata.level());
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
            let arg_string = format!("{}", record.args());
            let lvl = match record.level() {
                Level::Debug => CoreLogLevel::Debug,
                Level::Error => CoreLogLevel::Error,
                Level::Info => CoreLogLevel::Info,
                Level::Warn => CoreLogLevel::Warn,
                Level::Trace => CoreLogLevel::Trace,
            };

            //TODO: compare levels and continue only if required

            let lmts = CoreLogMessageThreadSafe {
                level: lvl,
                text: arg_string,
                time: Utc::now().timestamp(),
            };

            SimpleLogger::log_message_to_app(lmts);
        }
    }

    fn flush(&self) {}
}

//Impl used for tests
#[cfg(test)]
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // println!("metadata level : {}", metadata.level());
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} {} {}:{} - {}", Local::now().format("%H:%M:%S.%s"), record.level(), record.target(), record.line().unwrap_or(0), record.args());          
        }
    }

    fn flush(&self) {}
}

use crate::simple_logger;

#[test]
fn verify_test_macros() {
    // std::env::set_var("RUST_LOG", "trace");
    simple_logger::setup_with_level(LevelFilter::Debug);
    println!("Resulting level : {}", log::max_level());
    println!("STATIC_MAX_LEVEL : {}", log::STATIC_MAX_LEVEL);
    info!("first line");
    trace!("trace");
    // log::log!(Level::Trace, "Uja trace");
    // log::log!(Level::Debug, "Uja debug");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");
    assert_eq!(1,1)
}
