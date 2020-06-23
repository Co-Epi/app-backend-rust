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
    sys::{jboolean, jfloat, jint, jobject, jstring},
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
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_bootstrapCore(
    env: JNIEnv,
    _: JClass,
    db_path_j_string: JString,
    log_level_j_string: JString,
    log_coepi_only: jboolean,
    log_callback: jobject,
) -> jobject {
    init_log(&env, log_level_j_string, log_coepi_only, log_callback);

    let db_path_java_str = env.get_string(db_path_j_string).unwrap();
    let db_path_str = db_path_java_str.to_str().map_err(ServicesError::from);
    info!("Bootstrapping with db path: {:?}", db_path_str);
    let db_result = db_path_str.and_then(|path| bootstrap(path).map_err(ServicesError::from));
    info!("Bootstrapping result: {:?}", db_result);

    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_fetchNewReports(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    info!("Updating reports");
    // TODO error handling https://github.com/Co-Epi/app-backend-rust/issues/79
    let result = dependencies().reports_updater.fetch_new_reports().unwrap();
    info!("New reports: {:?}", result);

    let alerts_j_objects: Vec<jobject> = result
        .into_iter()
        .map(|alert| alert_to_jobject(alert, &env))
        .collect();

    let placeholder_alert_j_object = alert_to_jobject(placeholder_alert(), &env);

    let alerts_array = env
        .new_object_array(
            alerts_j_objects.len() as i32,
            "org/coepi/android/core/JniAlert",
            placeholder_alert_j_object,
        )
        .unwrap();

    for (index, alert_j_object) in alerts_j_objects.into_iter().enumerate() {
        env.set_object_array_element(alerts_array, index as i32, alert_j_object)
            .unwrap();
    }

    jni_obj_result(
        1,
        None,
        JObject::from(alerts_array),
        "org/coepi/android/core/JniAlertsArrayResult",
        "[Lorg/coepi/android/core/JniAlert;",
        &env,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_recordTcn(
    env: JNIEnv,
    _: JClass,
    tcn: JString,
) -> jobject {
    let tcn_java_str = env.get_string(tcn).unwrap();
    let tcn_str = tcn_java_str.to_str().map_err(ServicesError::from);

    let result = tcn_str.and_then(|tcn_str| dependencies().observed_tcn_processor.save(tcn_str));
    info!("Recording TCN result {:?}", result);
    jni_void_result(1, None, &env)
}

// NOTE: Returns directly success string
#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_generateTcn(
    env: JNIEnv,
    _: JClass,
) -> jstring {
    // TODO hex encoding in component, or send byte array directly?
    let tcn_hex = hex::encode(dependencies().tcn_keys.generate_tcn().0);
    info!("Generated TCN: {:?}", tcn_hex);

    let output_res = env.new_string(tcn_hex);
    let output = expect_log!(output_res, "Couldn't create java string");

    output.into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setSymptomIds(
    env: JNIEnv,
    _: JClass,
    ids: JString,
) -> jobject {
    let ids_java_str = env.get_string(ids).unwrap();
    let ids_str = ids_java_str.to_str().map_err(ServicesError::from);

    debug!("Setting symptom ids: {:?}", ids_str);

    let result = ids_str.and_then(|ids_str| {
        dependencies()
            .symptom_inputs_processor
            .set_symptom_ids(ids_str)
    });
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setCoughType(
    env: JNIEnv,
    _: JClass,
    cough_type: JString,
) -> jobject {
    let java_str = env.get_string(cough_type).unwrap();
    let str = java_str.to_str().map_err(ServicesError::from);

    debug!("Setting cough type: {:?}", str);
    let result = str.and_then(|cough_type_str| {
        dependencies()
            .symptom_inputs_processor
            .set_cough_type(cough_type_str)
    });
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setCoughDays(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    days: jint,
) -> jobject {
    let result = dependencies()
        .symptom_inputs_processor
        .set_cough_days(is_set == 1, days as u32);
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setCoughStatus(
    env: JNIEnv,
    _: JClass,
    cough_status: JString,
) -> jobject {
    let java_str = env.get_string(cough_status).unwrap();
    let str = java_str.to_str().map_err(ServicesError::from);

    info!("Setting cough status: {:?}", str);
    let result = str.and_then(|status_str| {
        dependencies()
            .symptom_inputs_processor
            .set_cough_status(status_str)
    });
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setBreathlessnessCause(
    env: JNIEnv,
    _: JClass,
    cause: JString,
) -> jobject {
    let java_str = env.get_string(cause).unwrap();
    let str = java_str.to_str().map_err(ServicesError::from);

    debug!("Setting breathlessness cause: {:?}", str);
    let result = str.and_then(|cause_str| {
        dependencies()
            .symptom_inputs_processor
            .set_breathlessness_cause(cause_str)
    });
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setFeverDays(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    days: jint,
) -> jobject {
    // TODO is_set jboolean
    // TODO assert is_set / days etc. in type's bounds, also iOS
    let result = dependencies()
        .symptom_inputs_processor
        .set_fever_days(is_set == 1, days as u32);
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setFeverTakenTemperatureToday(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    taken: jint,
) -> jobject {
    let result = dependencies()
        .symptom_inputs_processor
        .set_fever_taken_temperature_today(is_set == 1, taken == 1);
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setFeverTakenTemperatureSpot(
    env: JNIEnv,
    _: JClass,
    spot: JString,
) -> jobject {
    let java_str = env.get_string(spot).unwrap();
    let str = java_str.to_str().map_err(ServicesError::from);

    debug!("Setting temperature spot cause: {:?}", str);
    let result = str.and_then(|spot_str| {
        dependencies()
            .symptom_inputs_processor
            .set_fever_taken_temperature_spot(spot_str)
    });
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setFeverHighestTemperatureTaken(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    temp: jfloat,
) -> jobject {
    let result = dependencies()
        .symptom_inputs_processor
        .set_fever_highest_temperature_taken(is_set == 1, temp as f32);
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_setEarliestSymptomStartedDaysAgo(
    env: JNIEnv,
    _: JClass,
    is_set: jint,
    days: jint,
) -> jobject {
    let result = dependencies()
        .symptom_inputs_processor
        .set_earliest_symptom_started_days_ago(is_set == 1, days as u32);
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_clearSymptoms(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let result = dependencies().symptom_inputs_processor.clear();
    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_submitSymptoms(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let result = dependencies().symptom_inputs_processor.submit();
    jni_void_result(1, None, &env)
}

fn init_log(env: &JNIEnv, level_j_string: JString, coepi_only: jboolean, callback: jobject) -> i32 {
    let callback_wrapper = LogCallbackWrapperImpl {
        java_vm: env.get_java_vm().unwrap(),
        callback: env.new_global_ref(callback).unwrap(),
    };
    register_callback_internal(Box::new(callback_wrapper));

    let level_java_str = env.get_string(level_j_string).unwrap();
    let level_str = level_java_str.to_str().unwrap();
    let filter_level_res = LevelFilter::from_str(&level_str);
    let filter_level = expect_log!(filter_level_res, "Incorrect log level selected!");
    let _ = simple_logger::setup_logger(filter_level, coepi_only != 0);
    log::max_level() as i32
}

pub fn jni_void_result(status: i32, message: Option<&str>, env: &JNIEnv) -> jobject {
    let cls = env.find_class("org/coepi/android/core/JniVoidResult");

    let status_j_value = JValue::from(status);

    let msg = message.unwrap_or("");
    let msg_j_string = env.new_string(msg).unwrap();
    let msg_j_value = JValue::from(msg_j_string);

    let obj = env.new_object(
        cls.unwrap(),
        "(ILjava/lang/String;)V",
        &[status_j_value, msg_j_value],
    );

    obj.unwrap().into_inner()
}

pub fn jni_obj_result(
    status: i32,
    message: Option<&str>,
    obj: JObject,
    outer_class: &str,
    inner_class: &str,
    env: &JNIEnv,
) -> jobject {
    let cls = env.find_class(outer_class).unwrap();

    let status_j_value = JValue::from(status);

    let msg = message.unwrap_or("");

    let msg_j_string = env.new_string(msg).unwrap();
    let msg_j_value = JValue::from(msg_j_string);

    let obj = env.new_object(
        cls,
        format!("(ILjava/lang/String;{})V", inner_class),
        &[status_j_value, msg_j_value, JValue::from(obj)],
    );

    obj.unwrap().into_inner()
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
        let env = self.java_vm.attach_current_thread().unwrap();

        let level_j_value = JValue::from(level as i32);

        let text_j_string_res = env.new_string(text);
        let text_j_string = expect_log!(text_j_string_res, "Couldn't create java string!");

        let text_j_value = JValue::from(JObject::from(text_j_string));

        let res = env.call_method(
            self.callback.as_obj(),
            "log",
            "(ILjava/lang/String;)V",
            &[level_j_value, text_j_value],
        );
        expect_log!(res, "Couldn't call callback");
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
        contact_time: 0,
    }
}

pub fn alert_to_jobject(alert: Alert, env: &JNIEnv) -> jobject {
    let jni_public_report_class = env
        .find_class("org/coepi/android/core/JniPublicReport")
        .unwrap();

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
    );

    let jni_alert_class = env.find_class("org/coepi/android/core/JniAlert").unwrap();

    let id_j_string = env.new_string(alert.id).unwrap();
    let id_j_value = JValue::from(JObject::from(id_j_string));

    let earliest_time_j_value = JValue::from(alert.contact_time as i64);

    env.new_object(
        jni_alert_class,
        "(Ljava/lang/String;Lorg/coepi/android/core/JniPublicReport;J)V",
        &[
            id_j_value,
            JValue::from(jni_public_report_obj.unwrap()),
            earliest_time_j_value,
        ],
    )
    .unwrap()
    .into_inner()
}
