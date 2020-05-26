use once_cell::sync::OnceCell;
use persy::{Config, Persy, ValueMode};
use std::path::Path;
use tcn::TemporaryContactNumber;
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

fn u128_of_tcn(tcn: &TemporaryContactNumber) -> u128 {
    u128::from_le_bytes(tcn.0)
}

// maybe we don't care about this one?
// leaving it here in case I need it as the library evolves
// TODO: consider deleting
// fn cen_of_u128(u: u128) -> ContactEventNumber {
//     ContactEventNumber(u.to_le_bytes())
// }


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


fn all_stored_tcns() -> Res<Vec<u128>> {
    let mut out: Vec<u128> = Vec::new();

    let items = DB
        .get()
        .ok_or(DB_UNINIT)?
        .scan("tcn")?;

    for (_id,content) in items {
      let byte_array: [u8; 16] = byte_vec_to_16_byte_array(content);
      let tcn_bits: u128 = u128::from_le_bytes(byte_array);
      out.push(tcn_bits);
    }
    Ok(out)
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
