
use std::os::raw::{c_char};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::TCFType;
use serde::Serialize;

use crate::networking;

// Generic struct to return results to app
#[derive(Serialize)]
struct LibResult<T> {
  status: i32,
  data: Option<T>,
  error_message: Option<String>
}

#[no_mangle]
pub unsafe extern "C" fn get_reports(interval_number: u32, interval_length: u32) -> CFStringRef {
    println!("RUST: fetching reports for interval_number: {}, interval_length {}", interval_number, interval_length);

    let result = networking::get_reports(interval_number, interval_length);

    println!("RUST: Api returned: {:?}", result);

    let lib_result = match result {
      Ok(success) => LibResult { status: 1, data: Some(success), error_message: None },
      Err(e) => LibResult { status: 2, data: None, error_message: Some(e.to_string()) }
    };

    let lib_result_string = serde_json::to_string(&lib_result).unwrap();

    let cf_string = CFString::new(&lib_result_string);
    let cf_string_ref = cf_string.as_concrete_TypeRef();

    ::std::mem::forget(cf_string);

    return cf_string_ref;
}

#[no_mangle]
pub unsafe extern "C" fn post_report(c_report: *const c_char) -> CFStringRef {
  println!("RUST: posting report: {:?}", c_report);

  // TODO don't unwrap, use and handle result, handle
  let report = cstring_to_str(&c_report).unwrap();

  let result = networking::post_report(report.to_owned());

  let lib_result: LibResult<()> = match result {
    // TODO handle non 20x HTTP status
    Ok(success) => LibResult { status: 1, data: None, error_message: None },
    Err(e) => LibResult { status: 2, data: None, error_message: Some(e.to_string()) }
  };

  let lib_result_string = serde_json::to_string(&lib_result).unwrap();

  let cf_string = CFString::new(&lib_result_string);
  let cf_string_ref = cf_string.as_concrete_TypeRef();

  ::std::mem::forget(cf_string);

  return cf_string_ref;
}

// Convert C string to Rust string slice
unsafe fn cstring_to_str<'a>(cstring: &'a *const c_char) -> Option<&str> {
  if cstring.is_null() {
      return None;
  }

  let raw = ::std::ffi::CStr::from_ptr(*cstring);
  match raw.to_str() {
      Ok(s) => Some(s),
      Err(_) => None
  }
}
