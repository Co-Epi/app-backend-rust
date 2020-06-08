use crate::reports_updater::ObservedTcnProcessor;
use crate::tcn_ext::tcn_keys::TcnKeys;
use crate::{
    composition_root::COMP_ROOT,
    errors::ServicesError::{self},
    networking,
    simple_logger,
};
use crate::{init_db, reporting::symptom_inputs_manager::SymptomInputsProcessor};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use networking::TcnApi;
use serde::Serialize;
use std::os::raw::c_char;
use log::*;

// Generic struct to return results to app
// For convenience, status will be HTTP status codes
#[derive(Serialize)]
struct LibResult<T> {
    status: u16,
    data: Option<T>,
    error_message: Option<String>,
}

#[no_mangle]
pub unsafe extern "C" fn bootstrap_core(db_path: *const c_char) -> CFStringRef {
    let db_path_str = cstring_to_str(&db_path).unwrap();

    println!("RUST: bootstrapping with db path: {:?}", db_path_str);
    //TODO: Investigate using Box-ed logger
    let _ = simple_logger::init();
    let result = init_db(db_path_str).map_err(ServicesError::from);
    // println!("RUST: bootstrapping result: {:?}", result);
    info!("RUST: bootstrapping result: {:?}", result);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn fetch_new_reports() -> CFStringRef {
    info!("RUST: updating reports");

    let result = COMP_ROOT.reports_updater.fetch_new_reports();

    info!("RUST: new reports: {:?}", result);

    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn record_tcn(c_tcn: *const c_char) -> CFStringRef {
    // TODO don't unwrap, use and handle result, handle
    let tcn_str = cstring_to_str(&c_tcn).unwrap();
    info!("RUST: recording a TCN {:?}", c_tcn);
    let result = COMP_ROOT.observed_tcn_processor.save(tcn_str);
    info!("RUST: recording TCN result {:?}", result);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn generate_tcn() -> CFStringRef {
    // TODO hex encoding in component, or send byte array directly?
    let tcn_hex = hex::encode(COMP_ROOT.tcn_keys.generate_tcn().0);
    info!("RUST generated TCN: {:?}", tcn_hex);
    // info!("RUST generated TCN: {:?}", tcn_hex);

    let cf_string = CFString::new(&tcn_hex);
    let cf_string_ref = cf_string.as_concrete_TypeRef();

    ::std::mem::forget(cf_string);

    cf_string_ref
}

#[no_mangle]
pub unsafe extern "C" fn get_reports(interval_number: u32, interval_length: u32) -> CFStringRef {
    info!(
        "RUST: fetching reports for interval_number: {}, interval_length {}",
        interval_number, interval_length
    );

    let result = COMP_ROOT
        .api
        .get_reports(interval_number as u64, interval_length as u64);

    info!("RUST: Api returned: {:?}", result);

    let lib_result = match result {
        Ok(success) => LibResult {
            status: 200,
            data: Some(success),
            error_message: None,
        },
        Err(e) => LibResult {
            status: e.http_status,
            data: None,
            error_message: Some(e.to_string()),
        },
    };

    let lib_result_string = serde_json::to_string(&lib_result).unwrap();

    let cf_string = CFString::new(&lib_result_string);
    let cf_string_ref = cf_string.as_concrete_TypeRef();

    ::std::mem::forget(cf_string);

    return cf_string_ref;
}

fn to_result_str<T: Serialize>(result: Result<T, ServicesError>) -> CFStringRef {
    let lib_result = match result {
        Ok(success) => LibResult {
            status: 200,
            data: Some(success),
            error_message: None,
        },
        // TODO better error identification, using HTTP status for everything is weird.
        Err(e) => LibResult {
            status: 500,
            data: None,
            error_message: Some(e.to_string()),
        },
    };

    let lib_result_string = serde_json::to_string(&lib_result).unwrap();

    let cf_string = CFString::new(&lib_result_string);
    let cf_string_ref = cf_string.as_concrete_TypeRef();

    ::std::mem::forget(cf_string);

    return cf_string_ref;
}

#[no_mangle]
pub unsafe extern "C" fn set_symptom_ids(c_ids: *const c_char) -> CFStringRef {
    debug!("RUST: setting symptom ids: {:?}", c_ids);
    // TODO don't unwrap, use and handle result, handle
    let ids_str = cstring_to_str(&c_ids).unwrap();
    let result = COMP_ROOT.symptom_inputs_processor.set_symptom_ids(ids_str);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_cough_type(c_cough_type: *const c_char) -> CFStringRef {
    debug!("RUST: setting cough type: {:?}", c_cough_type);
    // TODO don't unwrap, use and handle result, handle
    let cough_type_str = cstring_to_str(&c_cough_type).unwrap();
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_cough_type(cough_type_str);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_cough_days(c_is_set: u8, c_days: u32) -> CFStringRef {
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_cough_days(c_is_set == 1, c_days);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_cough_status(c_status: *const c_char) -> CFStringRef {
    debug!("RUST: setting cough status: {:?}", c_status);
    // TODO don't unwrap, use and handle result, handle
    let status_str = cstring_to_str(&c_status).unwrap();
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_cough_status(status_str);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_breathlessness_cause(c_cause: *const c_char) -> CFStringRef {
    debug!("RUST: setting breathlessness cause: {:?}", c_cause);
    // TODO don't unwrap, use and handle result, handle
    let cause_str = cstring_to_str(&c_cause).unwrap();
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_breathlessness_cause(cause_str);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_fever_days(c_is_set: u8, c_days: u32) -> CFStringRef {
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_fever_days(c_is_set == 1, c_days);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_fever_taken_temperature_today(
    c_is_set: u8,
    c_taken: u8,
) -> CFStringRef {
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_fever_taken_temperature_today(c_is_set == 1, c_taken == 1);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_fever_taken_temperature_spot(c_cause: *const c_char) -> CFStringRef {
    debug!("RUST: setting temperature spot cause: {:?}", c_cause);
    // TODO don't unwrap, use and handle result, handle
    let spot_str = cstring_to_str(&c_cause).unwrap();
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_fever_taken_temperature_spot(spot_str);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_fever_highest_temperature_taken(
    c_is_set: u8,
    c_temp: f32,
) -> CFStringRef {
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_fever_highest_temperature_taken(c_is_set == 1, c_temp);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_earliest_symptom_started_days_ago(
    c_is_set: u8,
    c_days: u32,
) -> CFStringRef {
    let result = COMP_ROOT
        .symptom_inputs_processor
        .set_earliest_symptom_started_days_ago(c_is_set == 1, c_days);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn clear_symptoms() -> CFStringRef {
    let result = COMP_ROOT.symptom_inputs_processor.clear();
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn submit_symptoms() -> CFStringRef {
    let result = COMP_ROOT.symptom_inputs_processor.submit();
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn post_report(c_report: *const c_char) -> CFStringRef {
    info!("RUST: posting report: {:?}", c_report);

    // TODO don't unwrap, use and handle result, handle
    let report = cstring_to_str(&c_report).unwrap();

    let result = COMP_ROOT.api.post_report(report.to_owned());

    let lib_result: LibResult<()> = match result {
        Ok(_) => LibResult {
            status: 200,
            data: None,
            error_message: None,
        },
        Err(e) => LibResult {
            status: e.http_status,
            data: None,
            error_message: Some(e.to_string()),
        },
    };

    let lib_result_string = serde_json::to_string(&lib_result).unwrap();

    let cf_string = CFString::new(&lib_result_string);
    let cf_string_ref = cf_string.as_concrete_TypeRef();

    ::std::mem::forget(cf_string);

    return cf_string_ref;
}

// Convert C string to Rust string slice
pub unsafe fn cstring_to_str<'a>(cstring: &'a *const c_char) -> Option<&str> {
    if cstring.is_null() {
        return None;
    }

    let raw = ::std::ffi::CStr::from_ptr(*cstring);
    match raw.to_str() {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_reports() {
        unsafe {
            let res = get_reports(1, 21600);
            debug!("reports: {:?}", res);
            assert!(true);
        }
    }
}
