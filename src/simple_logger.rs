#[cfg(not(test))]
use crate::ios::ios_interface::{CoreLogLevel, CoreLogMessageThreadSafe, LOG_SENDER};
#[cfg(test)]
use chrono::Local;
#[cfg(not(test))]
use chrono::Utc;
use log::*;
use std::sync::Once;

static INIT: Once = Once::new();

//Boxed logger setup
pub fn setup_logger(level: LevelFilter, coepi_only: bool) {
    INIT.call_once(|| {
        println!("RUST : Logger level : {}", level);
        if coepi_only {
            println!("RUST : CoEpi logs only",);
            set_boxed_logger(Box::new(CoEpiLogger {}))
                .map(|()| log::set_max_level(level))
                .expect("Logger initialization failed!");
        } else {
            set_boxed_logger(Box::new(SimpleLogger {}))
                .map(|()| log::set_max_level(level))
                .expect("Logger initialization failed!");
        }
    })
}
//https://github.com/rust-lang/log/blob/efcc39c5217edae4f481b73357ca2f868bfe0a2c/test_max_level_features/main.rs#L10
fn set_boxed_logger(logger: Box<dyn Log>) -> Result<(), log::SetLoggerError> {
    log::set_logger(Box::leak(logger))
}

//Convenience fn
#[cfg(test)]
pub fn setup() {
    setup_logger(LevelFilter::Trace, false);
}

//Logs everything
pub struct SimpleLogger {}
//Logs CoEpi specific messages only
pub struct CoEpiLogger {}

#[cfg(not(test))]
macro_rules! log_prod {
    ($sel: ident, $record: ident) => {{
        if $sel.enabled($record.metadata()) {
            let arg_string = format!("{}", $record.args());
            let lvl = match $record.level() {
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
    }};
}

#[cfg(test)]
macro_rules! log_test {
    ($sel: ident, $record: ident)  => {
        if $sel.enabled($record.metadata()) {
            println!(
                "{} {} {}:{} - {}",
                Local::now().format("%H:%M:%S.%s"),
                $record.level(),
                $record.target(),
                $record.line().unwrap_or(0),
                $record.args()
            );
        }
    };
}

impl log::Log for CoEpiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level() && metadata.target().starts_with("coepi_core::")
    }
    #[cfg(not(test))]
    fn log(&self, record: &Record) {
        log_prod!(self, record);
        // if self.enabled(record.metadata()) {
        //     let arg_string = format!("{}", record.args());
        //     let lvl = match record.level() {
        //         Level::Debug => CoreLogLevel::Debug,
        //         Level::Error => CoreLogLevel::Error,
        //         Level::Info => CoreLogLevel::Info,
        //         Level::Warn => CoreLogLevel::Warn,
        //         Level::Trace => CoreLogLevel::Trace,
        //     };

        //     let lmts = CoreLogMessageThreadSafe {
        //         level: lvl,
        //         text: arg_string,
        //         time: Utc::now().timestamp(),
        //     };

        //     SimpleLogger::log_message_to_app(lmts);
        // }
    }
    #[cfg(test)]
    fn log(&self, record: &Record) {
        log_test!(self, record);
        // if self.enabled(record.metadata()) {
        //     println!(
        //         "{} {} {}:{} - {}",
        //         Local::now().format("%H:%M:%S.%s"),
        //         record.level(),
        //         record.target(),
        //         record.line().unwrap_or(0),
        //         record.args()
        //     );
        // }
    }

    fn flush(&self) {}
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


impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }
    #[cfg(not(test))]
    fn log(&self, record: &Record) {
        log_prod!(self, record);
        // if self.enabled(record.metadata()) {
        //     let arg_string = format!("{}", record.args());
        //     let lvl = match record.level() {
        //         Level::Debug => CoreLogLevel::Debug,
        //         Level::Error => CoreLogLevel::Error,
        //         Level::Info => CoreLogLevel::Info,
        //         Level::Warn => CoreLogLevel::Warn,
        //         Level::Trace => CoreLogLevel::Trace,
        //     };

        //     let lmts = CoreLogMessageThreadSafe {
        //         level: lvl,
        //         text: arg_string,
        //         time: Utc::now().timestamp(),
        //     };

        //     SimpleLogger::log_message_to_app(lmts);
        // }
    }

    #[cfg(test)]
    fn log(&self, record: &Record) {
        log_test!(self, record);
        // if self.enabled(record.metadata()) {
        //     println!(
        //         "{} {} {}:{} - {}",
        //         Local::now().format("%H:%M:%S.%s"),
        //         record.level(),
        //         record.target(),
        //         record.line().unwrap_or(0),
        //         record.args()
        //     );
        // }
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
