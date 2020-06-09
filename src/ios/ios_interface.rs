use crate::reports_updater::ObservedTcnProcessor;
use crate::tcn_ext::tcn_keys::TcnKeys;
use crate::{composition_root::COMP_ROOT, errors::ServicesError, networking, simple_logger};
use crate::{init_db, reporting::symptom_inputs_manager::SymptomInputsProcessor};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use log::*;
use networking::TcnApi;
use serde::Serialize;
use std::os::raw::c_char;

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
    let db_path_str = cstring_to_str(&db_path);

    println!("RUST: bootstrapping with db path: {:?}", db_path_str);
    //TODO: Investigate using Box-ed logger
    //TODO: let app set max_logging_level
    let _ = simple_logger::init();
    let result = db_path_str.and_then(|path| init_db(path).map_err(ServicesError::from));
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
    let tcn_str = cstring_to_str(&c_tcn);
    info!("RUST: recording a TCN {:?}", c_tcn);
    let result = tcn_str.and_then(|tcn_str| COMP_ROOT.observed_tcn_processor.save(tcn_str));
    info!("RUST: recording TCN result {:?}", result);
    return to_result_str(result);
}

// NOTE: Returns directly success string
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

    let lib_result_string =
        serde_json::to_string(&lib_result).unwrap_or_else(|_| fallback_error_result_str::<T>());

    let cf_string = CFString::new(&lib_result_string);
    let cf_string_ref = cf_string.as_concrete_TypeRef();

    ::std::mem::forget(cf_string);

    return cf_string_ref;
}

fn fallback_error_result_str<T: Serialize>() -> String {
    serde_json::to_string(&LibResult::<T> {
        status: 500,
        data: None,
        error_message: Some("Couldn't serialize result".to_owned()),
    })
    // unwrap: safe, since we are using a hardcoded value
    .unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn set_symptom_ids(c_ids: *const c_char) -> CFStringRef {
    debug!("RUST: setting symptom ids: {:?}", c_ids);
    let ids_str = cstring_to_str(&c_ids);
    let result =
        ids_str.and_then(|ids_str| COMP_ROOT.symptom_inputs_processor.set_symptom_ids(ids_str));
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_cough_type(c_cough_type: *const c_char) -> CFStringRef {
    debug!("RUST: setting cough type: {:?}", c_cough_type);
    let cough_type_str = cstring_to_str(&c_cough_type);
    let result = cough_type_str.and_then(|cough_type_str| {
        COMP_ROOT
            .symptom_inputs_processor
            .set_cough_type(cough_type_str)
    });
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
    let status_str = cstring_to_str(&c_status);
    let result = status_str.and_then(|status_str| {
        COMP_ROOT
            .symptom_inputs_processor
            .set_cough_status(status_str)
    });
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_breathlessness_cause(c_cause: *const c_char) -> CFStringRef {
    debug!("RUST: setting breathlessness cause: {:?}", c_cause);
    let cause_str = cstring_to_str(&c_cause);
    let result = cause_str.and_then(|cause_str| {
        COMP_ROOT
            .symptom_inputs_processor
            .set_breathlessness_cause(cause_str)
    });
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
    let spot_str = cstring_to_str(&c_cause);
    let result = spot_str.and_then(|spot_str| {
        COMP_ROOT
            .symptom_inputs_processor
            .set_fever_taken_temperature_spot(spot_str)
    });
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

    let report = cstring_to_str(&c_report);

    let result = report.and_then(|report| {
        COMP_ROOT
            .api
            .post_report(report.to_owned())
            .map_err(ServicesError::from)
    });

    return to_result_str(result);
}

// Convert C string to Rust string slice
pub unsafe fn cstring_to_str<'a>(cstring: &'a *const c_char) -> Result<&str, ServicesError> {
    if cstring.is_null() {
        return Err(ServicesError::FFIParameters("cstring is null".to_owned()));
    }

    let raw = ::std::ffi::CStr::from_ptr(*cstring);
    match raw.to_str() {
        Ok(s) => Ok(s),
        Err(e) => Err(ServicesError::FFIParameters(e.to_string())),
    }
}
