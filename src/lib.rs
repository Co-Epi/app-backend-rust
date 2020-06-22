#[macro_use]
extern crate serde_big_array;
use errors::Error;
use once_cell::sync::OnceCell;
use persy::{Config, Persy, ValueMode};
use std::path::Path;
mod composition_root;
mod errors;
mod networking;
mod preferences;
mod reporting;
mod reports_interval;
mod reports_updater;
mod simple_logger;
mod tcn_ext;

#[cfg(any(target_os = "ios", target_os = "macos"))]
mod ios;

#[cfg(target_os = "android")]
mod android;

pub type Res<T> = Result<T, Error>;

const CENS_BY_TS: &str = "cens by ts";

pub fn init_persy<P: AsRef<Path>>(p: P) -> Res<()> {
    let db = Persy::open_or_create_with(p, Config::new(), |db| {
        let mut tx = db.begin()?;
        tx.create_segment("tcn")?;
        tx.create_index::<i64, u128>(CENS_BY_TS, ValueMode::CLUSTER)?;
        tx.prepare_commit()?.commit()?;
        Ok(())
    })?;
    DB.set(db).map_err(|_| DB_ALREADY_INIT)?;
    Ok(())
}

const DB_ALREADY_INIT: &str = "DB failed to initalize";
pub const DB_UNINIT: &str = "DB not initialized";

// TODO since we're using DI put this in a dependency, to be consistent
pub static DB: OnceCell<Persy> = OnceCell::new();

// TODO refactor these (byte_vec_to) convertions or better way?

// TODO move to utils file or similar. Consider returning Result instead of panicking.
pub fn byte_vec_to_16_byte_array(bytes: Vec<u8>) -> [u8; 16] {
    let mut array = [0; 16];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes);
    array
}

pub fn byte_vec_to_24_byte_array(bytes: Vec<u8>) -> [u8; 24] {
    let mut array = [0; 24];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes);
    array
}

pub fn byte_vec_to_8_byte_array(bytes: Vec<u8>) -> [u8; 8] {
    let mut array = [0; 8];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes);
    array
}

pub fn byte_vec_to_32_byte_array(bytes: Vec<u8>) -> [u8; 32] {
    let mut array = [0; 32];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes);
    array
}

// TODO (deleting of TCNs not critical for now)
// pub fn delete_cens_between(start: i64, end: i64) -> Res<()> {
//     let db = DB.get().ok_or(DB_UNINIT)?;
//     let mut tx = db.begin()?;

//     let tsv = tx
//         .range::<i64, u128, _>(CENS_BY_TS, start..end)?
//         .map(|(ts, _)| ts)
//         .collect::<Vec<_>>();

//     for ts in tsv {
//         tx.remove::<i64, u128>(CENS_BY_TS, ts, None)?;
//     }

//     tx.prepare_commit()?.commit()?;
//     Ok(())
// }

// like Result.expect(), but it also logs the message + line number to the logger.
// This is needed for Android, which doesn't show stdout / panic messages.
// Using a macro temporarily. Ideally this should be in an extension of Result (see commented code below).
// With the later we can't get the caller line number at the moment.
// This will be possible when https://github.com/rust-lang/rust/pull/72445 is merged.
#[macro_export]
macro_rules! expect_log {
    ($res: ident, $msg: tt) => {{
        match $res {
            Ok(value) => value,
            Err(error) => {
                #[cfg(target_os = "android")]
                error!("Panic: line: {}, msg: {}, error:{:?}", line!(), $msg, error);
                panic!("{}: {:?}", $msg, error);
            }
        }
    }};
}

// trait ResultExt<T, E> {
//     fn expect_log(self, msg: &str) -> T;
// }
// impl<T, E> ResultExt<T, E> for Result<T, E>
// where
//     E: Debug,
// {
//     #[inline]
//     // https://github.com/rust-lang/rust/pull/72445
//     // #[track_caller]
//     fn expect_log(self, msg: &str) -> T {
//         match self {
//             Ok(t) => t,
//             Err(error) => {
//                 let msg = format!("{}: {:?}", msg, error);
//                 // Location::caller();
//                 #[cfg(target_os = "android")]
//                 error!("Panic: {}", msg);

//                 panic!(msg);
//             }
//         }
//     }
// }
