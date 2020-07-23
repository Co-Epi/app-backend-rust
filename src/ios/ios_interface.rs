use crate::reporting::symptom_inputs_manager::SymptomInputsProcessor;
use crate::tcn_ext::tcn_keys::TcnKeys;
use crate::{
    dependencies::{bootstrap, dependencies, DEPENDENCIES},
    errors::ServicesError,
    networking,
};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use log::*;
use networking::TcnApi;
use serde::Serialize;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
// use mpsc::Receiver;
use crate::simple_logger;
use crate::tcn_recording::observed_tcn_processor::ObservedTcnProcessor;
use simple_logger::{CoreLogLevel, CoreLogMessageThreadSafe, SENDER};
use std::os::raw::c_char;
use std::str::FromStr;

// Generic struct to return results to app
// For convenience, status will be HTTP status codes
#[derive(Serialize)]
struct LibResult<T> {
    status: u16,
    data: Option<T>,
    error_message: Option<String>,
}

#[no_mangle]
pub unsafe extern "C" fn setup_logger(level: CoreLogLevel, coepi_only: bool) -> i32 {
    let level_string = level.to_string();
    let filter_level = LevelFilter::from_str(&level_string).expect("Incorrect log level selected!");
    let _ = simple_logger::setup_logger(filter_level, coepi_only);
    log::max_level() as i32
}

#[no_mangle]
pub unsafe extern "C" fn bootstrap_core(
    db_path: *const c_char,
    level: CoreLogLevel,
    coepi_only: bool,
) -> CFStringRef {
    let level_string = level.to_string();
    let filter_level = LevelFilter::from_str(&level_string).expect("Incorrect log level selected!");
    let _ = simple_logger::setup_logger(filter_level, coepi_only);

    let db_path_str = cstring_to_str(&db_path);
    let result = db_path_str.and_then(|path| bootstrap(path).map_err(ServicesError::from));
    info!("Bootstrapping result: {:?}", result);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn fetch_new_reports() -> CFStringRef {
    info!("Updating reports");

    let result = dependencies().reports_updater.fetch_new_reports();

    info!("New reports: {:?}", result);

    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn record_tcn(c_tcn: *const c_char, distance: f32) -> CFStringRef {
    let tcn_str = cstring_to_str(&c_tcn);
    let result = tcn_str.and_then(|tcn_str| {
        dependencies()
            .observed_tcn_processor
            .save(tcn_str, distance)
    });
    return to_result_str(result);
}

// NOTE: Returns directly success string
#[no_mangle]
pub unsafe extern "C" fn generate_tcn() -> CFStringRef {
    // TODO hex encoding in component, or send byte array directly?
    let tcn_hex = hex::encode(
        DEPENDENCIES
            .get()
            .expect("Not bootstrapped")
            .tcn_keys
            .generate_tcn()
            .0,
    );
    info!("Generated TCN: {:?}", tcn_hex);

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
    debug!("Setting symptom ids: {:?}", c_ids);
    let ids_str = cstring_to_str(&c_ids);
    let result = ids_str.and_then(|ids_str| {
        dependencies()
            .symptom_inputs_processor
            .set_symptom_ids(ids_str)
    });
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_cough_type(c_cough_type: *const c_char) -> CFStringRef {
    debug!("Setting cough type: {:?}", c_cough_type);
    let cough_type_str = cstring_to_str(&c_cough_type);
    let result = cough_type_str.and_then(|cough_type_str| {
        dependencies()
            .symptom_inputs_processor
            .set_cough_type(cough_type_str)
    });
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_cough_days(c_is_set: u8, c_days: u32) -> CFStringRef {
    let result = dependencies()
        .symptom_inputs_processor
        .set_cough_days(c_is_set == 1, c_days);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_cough_status(c_status: *const c_char) -> CFStringRef {
    info!("Setting cough status: {:?}", c_status);
    let status_str = cstring_to_str(&c_status);
    let result = status_str.and_then(|status_str| {
        dependencies()
            .symptom_inputs_processor
            .set_cough_status(status_str)
    });
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_breathlessness_cause(c_cause: *const c_char) -> CFStringRef {
    debug!("Setting breathlessness cause: {:?}", c_cause);
    let cause_str = cstring_to_str(&c_cause);
    let result = cause_str.and_then(|cause_str| {
        dependencies()
            .symptom_inputs_processor
            .set_breathlessness_cause(cause_str)
    });
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_fever_days(c_is_set: u8, c_days: u32) -> CFStringRef {
    let result = dependencies()
        .symptom_inputs_processor
        .set_fever_days(c_is_set == 1, c_days);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_fever_taken_temperature_today(
    c_is_set: u8,
    c_taken: u8,
) -> CFStringRef {
    let result = dependencies()
        .symptom_inputs_processor
        .set_fever_taken_temperature_today(c_is_set == 1, c_taken == 1);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_fever_taken_temperature_spot(c_cause: *const c_char) -> CFStringRef {
    debug!("Setting temperature spot cause: {:?}", c_cause);
    let spot_str = cstring_to_str(&c_cause);
    let result = spot_str.and_then(|spot_str| {
        dependencies()
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
    let result = dependencies()
        .symptom_inputs_processor
        .set_fever_highest_temperature_taken(c_is_set == 1, c_temp);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn set_earliest_symptom_started_days_ago(
    c_is_set: u8,
    c_days: u32,
) -> CFStringRef {
    let result = dependencies()
        .symptom_inputs_processor
        .set_earliest_symptom_started_days_ago(c_is_set == 1, c_days);
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn clear_symptoms() -> CFStringRef {
    let result = dependencies().symptom_inputs_processor.clear();
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn submit_symptoms() -> CFStringRef {
    let result = dependencies().symptom_inputs_processor.submit();
    return to_result_str(result);
}

#[no_mangle]
pub unsafe extern "C" fn post_report(c_report: *const c_char) -> CFStringRef {
    info!("Posting report: {:?}", c_report);

    let report = cstring_to_str(&c_report);

    let result = report.and_then(|report| {
        dependencies()
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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CoreLogMessage {
    level: CoreLogLevel,
    text: CFStringRef,
    time: i64,
}

impl From<CoreLogMessageThreadSafe> for CoreLogMessage {
    fn from(lts: CoreLogMessageThreadSafe) -> Self {
        let cf_string = CFString::new(&lts.text);
        let cf_string_ref = cf_string.as_concrete_TypeRef();
        ::std::mem::forget(cf_string);
        CoreLogMessage {
            level: lts.level,
            text: cf_string_ref,
            time: lts.time,
        }
    }
}

pub trait LogCallback {
    fn call(&self, log_message: CoreLogMessage);
}

impl LogCallback for unsafe extern "C" fn(CoreLogMessage) {
    fn call(&self, log_message: CoreLogMessage) {
        unsafe {
            self(log_message);
        }
    }
}

fn register_log_callback_internal(callback: Box<dyn LogCallback>) {
    // Make callback implement Send (marker for thread safe, basically) https://doc.rust-lang.org/std/marker/trait.Send.html
    let log_callback = unsafe {
        std::mem::transmute::<Box<dyn LogCallback>, Box<dyn LogCallback + Send>>(callback)
    };

    // Create channel
    let (tx, rx): (
        Sender<CoreLogMessageThreadSafe>,
        Receiver<CoreLogMessageThreadSafe>,
    ) = mpsc::channel();

    // Save the sender in a static variable, which will be used to push elements to the callback
    unsafe {
        SENDER = Some(tx);
    }

    // Thread waits for elements pushed to SENDER and calls the callback
    thread::spawn(move || {
        for log_entry in rx.iter() {
            log_callback.call(log_entry.into());
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn trigger_logging_macros() -> i32 {
    debug!(target: "test_events", "CoEpi debug");
    trace!(target: "test_events", "CoEpi trace");
    info!(target: "test_events", "CoEpi info");
    warn!(target: "test_events", "CoEpi warn");
    error!(target: "test_events", "CoEpi error");

    1
}

#[no_mangle]
pub unsafe extern "C" fn register_log_callback(
    log_callback: unsafe extern "C" fn(CoreLogMessage),
) -> i32 {
    register_log_callback_internal(Box::new(log_callback));
    2
}
