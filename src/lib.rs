use std::os::raw::{c_char};
use std::ffi::{CString, CStr};

#[no_mangle]
pub extern fn rust_greeting(to: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(to) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string,
    };

    CString::new("Hello ".to_owned() + recipient).unwrap().into_raw()
}

/// Expose the JNI interface for android below
#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;

    use super::*;
    use self::jni::JNIEnv;
    use self::jni::objects::{JClass, JString};
    use self::jni::sys::{jstring};

    #[no_mangle]
    pub unsafe extern fn Java_org_coepi_android_api_NativeApi_getReports(env: JNIEnv, _: JClass, java_pattern: JString) -> jstring {
        // Our Java companion code might pass-in "world" as a string, hence the name.
        let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
        // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
        let world_ptr = CString::from_raw(world);
        let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

        output.into_inner()
    }

    #[no_mangle]
    pub unsafe extern fn Java_org_coepi_android_api_NativeApi_postReport(env: JNIEnv, _: JClass, java_pattern: JString) -> jstring {
        // Our Java companion code might pass-in "world" as a string, hence the name.
        let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
        // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
        let world_ptr = CString::from_raw(world);
        let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

        output.into_inner()
    }
}

/*
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
*/