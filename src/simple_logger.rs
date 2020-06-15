
use log::*;
use std::sync::Once;
#[cfg(not(test))]
use crate::ios::ios_interface::{CoreLogLevel, CoreLogMessageThreadSafe, LOG_SENDER};
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use chrono::Local;


static INIT: Once = Once::new();

//Boxed logger setup
pub fn setup_logger(level: LevelFilter, coepi_only: bool) {
    INIT.call_once(|| {
        println!("RUST : Logger level : {}", level);
        set_boxed_logger(Box::new(SimpleLogger {
            coepi_specific_logs_only: coepi_only,
        }))
        .map(|()| log::set_max_level(level))
        .expect("Logger initialization failed!");
    })
}
//https://github.com/rust-lang/log/blob/efcc39c5217edae4f481b73357ca2f868bfe0a2c/test_max_level_features/main.rs#L10
fn set_boxed_logger(logger: Box<dyn Log>) -> Result<(), log::SetLoggerError> {
    log::set_logger(Box::leak(logger))
}

pub fn setup() {
    setup_logger(LevelFilter::Trace, false);
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
        if self.coepi_specific_logs_only {
            metadata.level() <= log::max_level() && metadata.target().starts_with("coepi_core::")
        } else {
            metadata.level() <= log::max_level()
        }
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

        if self.coepi_specific_logs_only {
            metadata.level() <= log::max_level() && metadata.target().starts_with("coepi_core::")
        } else {
            metadata.level() <= log::max_level()
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


#[test]
fn verify_test_macros() {
    setup_logger(LevelFilter::Debug, false);
    println!("Resulting level : {}", log::max_level());
    println!("STATIC_MAX_LEVEL : {}", log::STATIC_MAX_LEVEL);
    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");
}
