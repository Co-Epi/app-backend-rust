use crate::reporting::symptom_inputs_manager::SymptomInputsProcessor;
use crate::reports_updater::ObservedTcnProcessor;
use crate::tcn_ext::tcn_keys::TcnKeys;
use crate::{
    composition_root::{bootstrap, dependencies},
    errors::ServicesError,
    expect_log,
    reporting::{
        public_report::{CoughSeverity, FeverSeverity, PublicReport},
        symptom_inputs::UserInput,
    },
    reports_interval::UnixTime,
    reports_updater::Alert,
    simple_logger,
};
use jni::{
    objects::{GlobalRef, JClass, JObject, JString, JValue},
    sys::{jboolean, jfloat, jint, jobject, jobjectArray, jstring},
    JNIEnv, JavaVM,
};
use log::*;
use mpsc::Receiver;
use simple_logger::{CoreLogLevel, CoreLogMessageThreadSafe, SENDER};
use std::str::FromStr;
use std::{
    sync::mpsc::{self, Sender},
    thread,
};

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_bootstrapCore(
    env: JNIEnv,
    _: JClass,
    db_path_j_string: JString,
    log_level_j_string: JString,
    log_coepi_only: jboolean,
    log_callback: jobject,
) -> jobject {
    bootstrap_core(
        &env,
        db_path_j_string,
        log_level_j_string,
        log_coepi_only,
        log_callback,
    )
    .to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_fetchNewReports(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let arr = fetch_new_reports(&env);

    match arr {
        Ok(a) => to_alerts_result_jobject(1, None, a, &env),
        Err(e) => {
            // If there's an error, return a JNI object with error status and an empty JNI array
            // TODO it may be possible to avoid empty array by making array in JniAlertsArrayResult optional
            let jni_error = e.to_jni_error();
            let empty_alerts_jobject_array = alerts_to_jobject_array(vec![], &env);
            // If the creation of the empty array fails, we've to crash, because we've to return an array.
            let empty_alerts_array = expect_log!(
                empty_alerts_jobject_array,
                "Critical: Failed instantiating empty error object"
            );
            to_alerts_result_jobject(
                jni_error.status,
                Some(jni_error.message.as_ref()),
                empty_alerts_array,
                &env,
            )
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_recordTcn(
    env: JNIEnv,
    _: JClass,
    tcn: JString,
    distance: jfloat,
) -> jobject {
    recordTcn(&env, tcn, distance).to_void_jni(&env)
}

// NOTE: Returns directly success string
#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_generateTcn(
    env: JNIEnv,
    _: JClass,
) -> jstring {
    // Maybe send byte array directly?
    let tcn_hex = hex::encode(dependencies().tcn_keys.generate_tcn().0);
    info!("Generated TCN: {:?}", tcn_hex);

    let output_res = env.new_string(tcn_hex);
    // Unclear about when new_string can return Error (TODO), and there's no meaningful handling in the app, so for now crash
    let output = expect_log!(output_res, "Couldn't create java string");

    output.into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setSymptomIds(
    env: JNIEnv,
    _: JClass,
    ids: JString,
) -> jobject {
    set_symptom_ids(&env, ids).to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setCoughType(
    env: JNIEnv,
    _: JClass,
    cough_type: JString,
) -> jobject {
    set_cough_type(&env, cough_type).to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setCoughDays(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    days: jint,
) -> jobject {
    dependencies()
        .symptom_inputs_processor
        .set_cough_days(is_set == 1, days as u32)
        .to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setCoughStatus(
    env: JNIEnv,
    _: JClass,
    cough_status: JString,
) -> jobject {
    set_cough_status(&env, cough_status).to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setBreathlessnessCause(
    env: JNIEnv,
    _: JClass,
    cause: JString,
) -> jobject {
    set_breathlessness_cause(&env, cause).to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setFeverDays(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    days: jint,
) -> jobject {
    // TODO is_set jboolean
    // TODO assert is_set / days etc. in type's bounds, also iOS
    dependencies()
        .symptom_inputs_processor
        .set_fever_days(is_set == 1, days as u32)
        .to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setFeverTakenTemperatureToday(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    taken: jint,
) -> jobject {
    dependencies()
        .symptom_inputs_processor
        .set_fever_taken_temperature_today(is_set == 1, taken == 1)
        .to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setFeverTakenTemperatureSpot(
    env: JNIEnv,
    _: JClass,
    spot: JString,
) -> jobject {
    set_fever_taken_temperature_spot(&env, spot).to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setFeverHighestTemperatureTaken(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    temp: jfloat,
) -> jobject {
    dependencies()
        .symptom_inputs_processor
        .set_fever_highest_temperature_taken(is_set == 1, temp as f32)
        .to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_setEarliestSymptomStartedDaysAgo(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    days: jint,
) -> jobject {
    dependencies()
        .symptom_inputs_processor
        .set_earliest_symptom_started_days_ago(is_set == 1, days as u32)
        .to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_clearSymptoms(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    dependencies()
        .symptom_inputs_processor
        .clear()
        .to_void_jni(&env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_submitSymptoms(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    dependencies()
        .symptom_inputs_processor
        .submit()
        .to_void_jni(&env)
}

fn bootstrap_core(
    env: &JNIEnv,
    db_path_j_string: JString,
    log_level_j_string: JString,
    log_coepi_only: jboolean,
    log_callback: jobject,
) -> Result<(), ServicesError> {
    init_log(&env, log_level_j_string, log_coepi_only, log_callback);

    let db_path_java_str = env.get_string(db_path_j_string)?;
    let db_path_str = db_path_java_str.to_str()?;

    info!("Bootstrapping with db path: {:?}", db_path_str);
    let db_result = bootstrap(db_path_str)?;
    info!("Bootstrapping result: {:?}", db_result);

    Ok(())
}

fn fetch_new_reports(env: &JNIEnv) -> Result<jobjectArray, ServicesError> {
    info!("Updating reports");
    let result = dependencies().reports_updater.fetch_new_reports()?;
    info!("New reports: {:?}", result);

    alerts_to_jobject_array(result, &env)
}

fn recordTcn(env: &JNIEnv, tcn: JString, distance: jfloat) -> Result<(), ServicesError> {
    let tcn_java_str = env.get_string(tcn)?;
    let tcn_str = tcn_java_str.to_str()?;

    let result = dependencies()
        .observed_tcn_processor
        .save(tcn_str, distance as f32);
    info!("Recording TCN result {:?}", result);

    result
}

fn set_symptom_ids(env: &JNIEnv, ids: JString) -> Result<(), ServicesError> {
    let java_str = env.get_string(ids)?;
    let ids_str = java_str.to_str()?;

    debug!("Setting symptom ids: {:?}", ids_str);

    dependencies()
        .symptom_inputs_processor
        .set_symptom_ids(ids_str)
}

fn set_cough_type(env: &JNIEnv, cough_type: JString) -> Result<(), ServicesError> {
    let java_str = env.get_string(cough_type)?;
    let cough_type_str = java_str.to_str()?;

    debug!("Setting cough type: {:?}", cough_type_str);

    dependencies()
        .symptom_inputs_processor
        .set_cough_type(cough_type_str)
}

fn set_cough_status(env: &JNIEnv, cough_status: JString) -> Result<(), ServicesError> {
    let java_str = env.get_string(cough_status)?;
    let str = java_str.to_str()?;

    dependencies()
        .symptom_inputs_processor
        .set_cough_status(str)
}

fn set_breathlessness_cause(env: &JNIEnv, cause: JString) -> Result<(), ServicesError> {
    let java_str = env.get_string(cause)?;
    let str = java_str.to_str()?;

    dependencies()
        .symptom_inputs_processor
        .set_breathlessness_cause(str)
}

fn set_fever_taken_temperature_spot(env: &JNIEnv, spot: JString) -> Result<(), ServicesError> {
    let java_str = env.get_string(spot)?;
    let str = java_str.to_str()?;

    debug!("Setting temperature spot cause: {:?}", str);
    dependencies()
        .symptom_inputs_processor
        .set_fever_taken_temperature_spot(str)
}

fn to_alerts_result_jobject(
    status: i32,
    message: Option<&str>,
    alerts: jobjectArray,
    env: &JNIEnv,
) -> jobject {
    jni_obj_result(
        status,
        message,
        JObject::from(alerts),
        "org/coepi/core/jni/JniAlertsArrayResult",
        "[Lorg/coepi/core/jni/JniAlert;",
        &env,
    )
}

fn alerts_to_jobject_array(
    alerts: Vec<Alert>,
    env: &JNIEnv,
) -> Result<jobjectArray, ServicesError> {
    let alerts_j_objects_res: Result<Vec<jobject>, ServicesError> = alerts
        .into_iter()
        .map(|alert| alert_to_jobject(alert, &env))
        .collect();

    let alerts_j_objects: Vec<jobject> = alerts_j_objects_res?;

    let placeholder_alert_j_object = alert_to_jobject(placeholder_alert(), &env)?;

    let alerts_array = env.new_object_array(
        alerts_j_objects.len() as i32,
        "org/coepi/core/jni/JniAlert",
        placeholder_alert_j_object,
    )?;

    for (index, alert_j_object) in alerts_j_objects.into_iter().enumerate() {
        env.set_object_array_element(alerts_array, index as i32, alert_j_object)?;
    }

    Ok(alerts_array)
}

fn init_log(env: &JNIEnv, level_j_string: JString, coepi_only: jboolean, callback: jobject) -> i32 {
    match (env.get_java_vm(), env.new_global_ref(callback)) {
        (Ok(java_vm), Ok(callback_global_ref)) => {
            let callback_wrapper = LogCallbackWrapperImpl {
                java_vm,
                callback: callback_global_ref,
            };
            register_callback_internal(Box::new(callback_wrapper));

            let level_java_str = env.get_string(level_j_string).unwrap();
            let level_str = level_java_str.to_str().unwrap();
            let filter_level_res = LevelFilter::from_str(&level_str);
            let filter_level = expect_log!(filter_level_res, "Incorrect log level selected!");
            let _ = simple_logger::setup_logger(filter_level, coepi_only != 0);
            log::max_level() as i32
        }

        // Note: These println will not show on Android, as LogCat doesn't show stdout / stderr.
        // panic will also not show anything useful, so there doesn't seem to be a point in crashing here.
        (Ok(_), Err(e)) => {
            println!("Couldn't initialize JNI env: {:?}", e);
            -1
        }
        (Err(e), Ok(_)) => {
            println!("Couldn't initialize vm: {:?}", e);
            -1
        }
        (Err(vm_e), Err(env_e)) => {
            println!(
                "Couldn't initialize JNI env: {:?} and vm: {:?}",
                vm_e, env_e
            );
            -1
        }
    }
}

pub fn jni_void_result(status: i32, message: Option<&str>, env: &JNIEnv) -> jobject {
    let cls_res = env.find_class("org/coepi/core/jni/JniVoidResult");

    let status_j_value = JValue::from(status);

    let msg = message.unwrap_or("");
    let msg_j_string_res = env.new_string(msg);

    // If we can't create a result to send to JNI, we only can crash
    let msg_j_string = expect_log!(msg_j_string_res, "Couldn't create JNI msg string");

    let msg_j_value = JValue::from(msg_j_string);

    // If we can't create a result to send to JNI, we only can crash
    let cls = expect_log!(cls_res, "Couldn't create JNI result class");

    let obj = env.new_object(
        cls,
        "(ILjava/lang/String;)V",
        &[status_j_value, msg_j_value],
    );

    let res = obj;
    // If we can't create a result to send to JNI, we only can crash
    expect_log!(res, "Couldn't create JNI result object").into_inner()
}

pub fn jni_obj_result(
    status: i32,
    message: Option<&str>,
    obj: JObject,
    outer_class: &str,
    inner_class: &str,
    env: &JNIEnv,
) -> jobject {
    let cls_res = env.find_class(outer_class);

    let status_j_value = JValue::from(status);

    let msg = message.unwrap_or("");

    let msg_j_string_res = env.new_string(msg);
    // If we can't create a result to send to JNI, we only can crash
    let msg_j_string = expect_log!(msg_j_string_res, "Couldn't create JNI msg string");
    let msg_j_value = JValue::from(msg_j_string);

    // If we can't create a result to send to JNI, we only can crash
    let cls = expect_log!(cls_res, "Couldn't create JNI result object");

    let obj = env.new_object(
        cls,
        format!("(ILjava/lang/String;{})V", inner_class),
        &[status_j_value, msg_j_value, JValue::from(obj)],
    );

    // If we can't create a result to send to JNI, we only can crash
    expect_log!(obj, "Couldn't create JNI result object").into_inner()
}

trait LogCallbackWrapper {
    fn call(&self, level: CoreLogLevel, text: String);
}

struct LogCallbackWrapperImpl {
    // The callback passed from Android is a local reference: only valid during the method call.
    // To store it, we need to put it in a global reference.
    // See https://developer.android.com/training/articles/perf-jni#local-and-global-references
    callback: GlobalRef,

    // We need JNIEnv to call the callback.
    // JNIEnv is valid only in the same thread, so we have to store the vm instead, and use it to get
    // a JNIEnv for the current thread.
    // See https://developer.android.com/training/articles/perf-jni#javavm-and-jnienvb
    java_vm: JavaVM,
}

impl LogCallbackWrapper for LogCallbackWrapperImpl {
    fn call(&self, level: CoreLogLevel, text: String) {
        match self.java_vm.attach_current_thread() {
            Ok(env) => self.call(level, text, &env),
            // The Android LogCat will not show this, but for consistency or testing with non-Android JNI.
            // Note that if we panic, LogCat will also not show a message, or location.
            // TODO consider writing to file. Otherwise it's impossible to notice this.
            Err(e) => println!(
                "Couldn't get env: Can't send log: level: {}, text: {}, e: {}",
                level, text, e
            ),
        }
    }
}

impl LogCallbackWrapperImpl {
    fn call(&self, level: CoreLogLevel, text: String, env: &JNIEnv) {
        let level_j_value = JValue::from(level as i32);

        let text_j_string_res = env.new_string(text.clone());
        let text_j_string = expect_log!(text_j_string_res, "Couldn't create java string!");

        let text_j_value = JValue::from(JObject::from(text_j_string));

        let res = env.call_method(
            self.callback.as_obj(),
            "log",
            "(ILjava/lang/String;)V",
            &[level_j_value, text_j_value],
        );

        // The Android LogCat will not show this, but for consistency or testing with non-Android JNI
        // Note that if we panic, LogCat will also not show a message, or location.
        // TODO consider writing to file. Otherwise it's impossible to notice this.
        if let Err(e) = res {
            println!(
                "Calling callback failed: error: {:?}, level: {}, text: {}",
                e, level, text,
            )
        }
    }
}

fn register_callback_internal(callback: Box<dyn LogCallbackWrapper>) {
    // Make callback implement Send (marker for thread safe, basically) https://doc.rust-lang.org/std/marker/trait.Send.html
    let log_callback = unsafe {
        std::mem::transmute::<Box<dyn LogCallbackWrapper>, Box<dyn LogCallbackWrapper + Send>>(
            callback,
        )
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
            log_callback.call(log_entry.level, log_entry.text.into());
        }
    });
}

// To prefill the JNI array (TODO can this be skipped?)
fn placeholder_alert() -> Alert {
    let report = PublicReport {
        report_time: UnixTime { value: 0 },
        earliest_symptom_time: UserInput::Some(UnixTime { value: 0 }),
        fever_severity: FeverSeverity::None,
        cough_severity: CoughSeverity::None,
        breathlessness: false,
        muscle_aches: false,
        loss_smell_or_taste: false,
        diarrhea: false,
        runny_nose: false,
        other: false,
        no_symptoms: false,
    };

    Alert {
        id: "0".to_owned(),
        report,
        contact_start: 0,
        contact_end: 0,
        min_distance: 0.0,
        avg_distance: 0.0,
    }
}

pub fn alert_to_jobject(alert: Alert, env: &JNIEnv) -> Result<jobject, ServicesError> {
    let jni_public_report_class = env.find_class("org/coepi/core/jni/JniPublicReport")?;

    let report_time_j_value = JValue::from(alert.report.report_time.value as i64);

    let earliest_time = match &alert.report.earliest_symptom_time {
        UserInput::Some(time) => time.value as i64,
        UserInput::None => -1,
    };
    let earliest_time_j_value = JValue::from(earliest_time);

    let fever_severity = match &alert.report.fever_severity {
        FeverSeverity::None => 0,
        FeverSeverity::Mild => 1,
        FeverSeverity::Serious => 2,
    };
    let fever_severity_j_value = JValue::from(fever_severity);

    let cough_severity = match &alert.report.cough_severity {
        CoughSeverity::None => 0,
        CoughSeverity::Existing => 1,
        CoughSeverity::Wet => 2,
        CoughSeverity::Dry => 3,
    };
    let cough_severity_j_value = JValue::from(cough_severity);

    let breathlessness_j_value = JValue::from(alert.report.breathlessness);
    let muscle_aches_j_value = JValue::from(alert.report.muscle_aches);
    let loss_smell_or_taste_j_value = JValue::from(alert.report.loss_smell_or_taste);
    let diarrhea_j_value = JValue::from(alert.report.diarrhea);
    let runny_nose_j_value = JValue::from(alert.report.runny_nose);
    let other_j_value = JValue::from(alert.report.other);
    let no_symptoms_j_value = JValue::from(alert.report.no_symptoms);

    let jni_public_report_obj = env.new_object(
        jni_public_report_class,
        "(JJIIZZZZZZZ)V",
        &[
            report_time_j_value,
            earliest_time_j_value,
            fever_severity_j_value,
            cough_severity_j_value,
            breathlessness_j_value,
            muscle_aches_j_value,
            loss_smell_or_taste_j_value,
            diarrhea_j_value,
            runny_nose_j_value,
            other_j_value,
            no_symptoms_j_value,
        ],
    )?;

    let jni_alert_class = env.find_class("org/coepi/core/jni/JniAlert")?;

    let id_j_string = env.new_string(alert.id)?;
    let id_j_value = JValue::from(JObject::from(id_j_string));

    let contact_start_j_value = JValue::from(alert.contact_start as i64);
    let contact_end_j_value = JValue::from(alert.contact_end as i64);
    let min_distance_j_value = JValue::from(alert.min_distance);
    let avg_distance_j_value = JValue::from(alert.avg_distance);

    let result: Result<jobject, jni::errors::Error> = env
        .new_object(
            jni_alert_class,
            "(Ljava/lang/String;Lorg/coepi/core/jni/JniPublicReport;JJFF)V",
            &[
                id_j_value,
                JValue::from(jni_public_report_obj),
                contact_start_j_value,
                contact_end_j_value,
                min_distance_j_value,
                avg_distance_j_value,
            ],
        )
        .map(|o| o.into_inner());

    result.map_err(ServicesError::from)
}

trait ResultExt<T, ServicesError> {
    fn to_void_jni(self, env: &JNIEnv) -> jobject;
}
impl<T> ResultExt<T, ServicesError> for Result<T, ServicesError> {
    fn to_void_jni(self, env: &JNIEnv) -> jobject {
        match self {
            Ok(_) => jni_void_result(1, None, &env),
            Err(error) => {
                let jni_error = error.to_jni_error();
                jni_void_result(jni_error.status, Some(jni_error.message.as_ref()), &env)
            }
        }
    }
}

trait JniErrorMappable {
    fn to_jni_error(&self) -> JniError;
}

impl JniErrorMappable for ServicesError {
    fn to_jni_error(&self) -> JniError {
        match self {
            ServicesError::Networking(networking_error) => JniError {
                status: 2,
                message: format!("{:?}", networking_error),
            },
            ServicesError::Error(error) => JniError {
                status: 3,
                message: format!("{:?}", error),
            },
            ServicesError::FFIParameters(msg) => JniError {
                status: 4,
                message: msg.to_owned(),
            },
            ServicesError::General(msg) => JniError {
                status: 5,
                message: msg.to_owned(),
            },
            ServicesError::NotFound => JniError {
                status: 6,
                message: "Not found".to_owned(),
            },
        }
    }
}

struct JniError {
    status: i32,
    message: String,
}
