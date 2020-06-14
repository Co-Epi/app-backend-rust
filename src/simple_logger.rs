use crate::ios::ffi_for_sanity_tests::{CoreLogLevel, CoreLogMessageThreadSafe, LOG_SENDER};
// use log::{Level, Metadata, Record};
// use log::{LevelFilter, SetLoggerError};
use chrono::{Local, Utc};
use log::*;
use std::sync::Once;

// #[cfg(feature = "std")]
// use log::set_boxed_logger;

static LOGGER: SimpleLogger = SimpleLogger {
    coepi_specific_logs_only: false,
};

static LOGGER_COEPI: SimpleLogger = SimpleLogger {
    coepi_specific_logs_only: true,
};

static INIT: Once = Once::new();

pub fn setup_with_level_and_target(level: LevelFilter, coepi_only: bool) {
    INIT.call_once(|| {
        println!("RUST : Logger level : {}", level);
        if coepi_only {
            log::set_logger(&LOGGER_COEPI)
                .map(|()| log::set_max_level(level))
                .expect("Logger initialization failed!");
        } else {
            log::set_logger(&LOGGER)
                .map(|()| log::set_max_level(level))
                .expect("Logger initialization failed!");
        }
    });
}

//Boxed logger setup
pub fn setup_boxed(level: LevelFilter, coepi_only: bool) {
    INIT.call_once(|| {
        println!("RUST : Logger level : {}", level);
        if coepi_only {
            println!("RUST : CoEpi Logs Only");
            set_boxed_logger(Box::new(SimpleLogger {
                coepi_specific_logs_only: true,
            }))
            .map(|()| log::set_max_level(level))
            .expect("Logger initialization failed!");
        } else {
            set_boxed_logger(Box::new(SimpleLogger {
                coepi_specific_logs_only: false,
            }))
            .map(|()| log::set_max_level(level))
            .expect("Logger initialization failed!");
        }
    })
}
//https://github.com/rust-lang/log/blob/efcc39c5217edae4f481b73357ca2f868bfe0a2c/test_max_level_features/main.rs#L10
fn set_boxed_logger(logger: Box<Log>) -> Result<(), log::SetLoggerError> {
    log::set_logger(Box::leak(logger))
}

pub fn setup_with_level(level: LevelFilter) {
    // Guaranteed to be executed only once (even if called multiple times).

    INIT.call_once(|| {
        println!("RUST : Logger level : {}", level);
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(level))
            .expect("Logger initialization failed!");
    });
}

pub fn setup() {
    setup_with_level(LevelFilter::Trace);
}

pub struct SimpleLogger {
    coepi_specific_logs_only: bool,
}

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
            // println!("{} - {}", record.level(), record.args());
            let arg_string = format!("{}", record.args());
            let lvl = match record.level() {
                Level::Debug => CoreLogLevel::Debug,
                Level::Error => CoreLogLevel::Error,
                Level::Info => CoreLogLevel::Info,
                Level::Warn => CoreLogLevel::Warn,
                Level::Trace => CoreLogLevel::Trace,
            };

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
        // println!("metadata target : {}", metadata.target());
        // let tgt = metadata.target();
        // let mtc = tgt.starts_with("coepi_core::");
        // println!("Match: {}", mtc);
        let level_threshhold = metadata.level() <= log::max_level();

        // let sl = SimpleLogger::from(logger());

        if (self.coepi_specific_logs_only) {
            level_threshhold && metadata.target().starts_with("coepi_core::")
        } else {
            level_threshhold
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{} {} {}:{} - {}",
                Local::now().format("%H:%M:%S.%s"),
                record.level(),
                record.target(),
                record.line().unwrap_or(0),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

use crate::simple_logger;

#[test]
fn verify_test_macros() {
    simple_logger::setup_with_level(LevelFilter::Debug);
    println!("Resulting level : {}", log::max_level());
    println!("STATIC_MAX_LEVEL : {}", log::STATIC_MAX_LEVEL);
    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");
    assert_eq!(1, 1)
}
