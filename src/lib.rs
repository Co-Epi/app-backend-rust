#[macro_use]
extern crate serde_big_array;
use once_cell::sync::OnceCell;
use persy::{Config, Persy, ValueMode};
use std::path::Path;
use errors::Error;
mod networking;
mod ios;
mod reports_interval;
mod reports_updater;
mod composition_root;
mod reporting;
mod errors;
mod preferences;
mod tcn_ext;
mod simple_logger;

pub type Res<T> = Result<T, Error>;

const CENS_BY_TS: &str = "cens by ts";

pub fn init_db<P: AsRef<Path>>(p: P) -> Res<()> {
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

    // use log::*;

    // #[test]
    // fn a_0(){
    //     //Lets initialize the logger in a test that will be first to run :|
    //     let logger = simple_logger::init();
    //     info!("Logger:{:?}", logger);
    //     warn!("Logger has been initialized: {}", logger.is_ok());
    //     assert_eq!(true, logger.is_ok());
    // }
   
  