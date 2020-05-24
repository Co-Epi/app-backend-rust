
use std::os::raw::{c_char};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::TCFType;
use serde::Serialize;

use crate::{composition_root::COMP_ROOT, networking, reporting::symptom_inputs::{SymptomInputsSubmitter, SymptomInputs}, errors::ServicesError::{Networking, Error}};
use networking::TcnApi;

// Generic struct to return results to app
// For convenience, status will be HTTP status codes 
#[derive(Serialize)]
struct LibResult<T> {
  status: u16,
  data: Option<T>,
  error_message: Option<String>
}

#[no_mangle]
pub unsafe extern "C" fn fetch_new_reports() -> CFStringRef {
  println!("RUST: updating reports");

  let result = COMP_ROOT.reports_updater.fetch_new_reports();

  println!("RUST: new reports: {:?}", result);

  let lib_result = match result {
    Ok(success) => LibResult { status: 200, data: Some(success), error_message: None },
    // TODO better error identification, using HTTP status for everything is weird.
    Err(e) => LibResult { status: 500, data: None, error_message: Some(e.to_string()) }
  };

  let lib_result_string = serde_json::to_string(&lib_result).unwrap();

  let cf_string = CFString::new(&lib_result_string);
  let cf_string_ref = cf_string.as_concrete_TypeRef();

  ::std::mem::forget(cf_string);

  return cf_string_ref;
}

#[no_mangle]
pub unsafe extern "C" fn get_reports(interval_number: u32, interval_length: u32) -> CFStringRef {
    println!("RUST: fetching reports for interval_number: {}, interval_length {}", interval_number, interval_length);

    let result = COMP_ROOT.api.get_reports(interval_number as u64, interval_length as u64);

    println!("RUST: Api returned: {:?}", result);

    let lib_result = match result {
      Ok(success) => LibResult { status: 200, data: Some(success), error_message: None },
      Err(e) => LibResult { status: e.http_status, data: None, error_message: Some(e.to_string()) }
    };

    let lib_result_string = serde_json::to_string(&lib_result).unwrap();

    let cf_string = CFString::new(&lib_result_string);
    let cf_string_ref = cf_string.as_concrete_TypeRef();

    ::std::mem::forget(cf_string);

    return cf_string_ref;
}

#[no_mangle]
pub unsafe extern "C" fn submit_symptoms(c_report: *const c_char) -> CFStringRef {
  println!("RUST: posting report: {:?}", c_report);

  // TODO don't unwrap, use and handle result, handle
  let report = cstring_to_str(&c_report).unwrap();

  let inputs: Result<SymptomInputs, serde_json::Error> = serde_json::from_str(report);

  let lib_result: LibResult<()> = match inputs {
    Ok(inputs) => {
      let result = COMP_ROOT.symptom_inputs_submitter.submit_inputs(inputs);
      match result {
        Ok(_) => LibResult { status: 200, data: None, error_message: None },
        Err(e) => match e {
          Networking(e) =>
            LibResult { status: e.http_status, data: None, error_message: Some(e.to_string()) },
          Error(e) => 
            LibResult { status: 500, data: None, error_message: Some(e.to_string()) }
        }
      }
    },
    Err(e) =>
      LibResult { status: 400, data: None, error_message: Some(e.to_string()) }
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

  let result = COMP_ROOT.api.post_report(report.to_owned());

  let lib_result: LibResult<()> = match result {
    Ok(_) => LibResult { status: 200, data: None, error_message: None },
    Err(e) => LibResult { status: e.http_status, data: None, error_message: Some(e.to_string()) }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_reports() {
    unsafe {
      let res = get_reports(1, 21600);
      println!("reports: {:?}", res);
      assert!(true);
    }
  }
}
