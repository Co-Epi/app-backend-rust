
use std::os::raw::{c_char};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::TCFType;
use serde_json::Value;

use crate::networking;

#[no_mangle]
pub unsafe extern "C" fn get_reports(interval_number: u32, interval_length: u32) -> CFStringRef {
    println!("RUST: fetching reports for interval_number: {}, interval_length {}", interval_number, interval_length);

    let result = networking::get_reports(interval_number, interval_length);

    println!("RUST: Api returned: {:?}", result);

    let result_string = match result {
      Ok(reports) => {
        println!("Get reports success: {:?}", reports);
        // TODO types / protocol to communicate with app. For now we could use strings (JSON).
        let json_value: Value = reports.into();
        // TODO (reqwest::Error { kind: Decode, source: Error("invalid type: integer `1`, expected a sequence", line: 1, column: 1) })
        format!("{}", json_value)
      },
      Err(e) => format!("ERROR posting report: {}", e)
    };

    let cf_string = CFString::new(&result_string);
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

  let result_string = match result {
    Ok(response) => {
      println!("Post report success: {:?}", response);
      "ok".to_owned()
    },
    Err(e) => format!("ERROR posting report: {}", e)
  };

  let cf_string = CFString::new(&result_string);
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
