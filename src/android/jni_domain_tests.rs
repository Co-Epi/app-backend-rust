use super::android_interface::{jni_obj_result, jni_void_result};
use crate::{
    composition_root::COMP_ROOT,
    errors::ServicesError,
    init_db,
    reporting::{
        public_report::{CoughSeverity, FeverSeverity, PublicReport},
        symptom_inputs::UserInput,
    },
    reports_interval::UnixTime,
    reports_updater::Alert,
    simple_logger,
};
use jni::{
    objects::{JClass, JObject, JString, JValue},
    sys::{jboolean, jobject, jobjectArray},
    JNIEnv,
};
use log::{info, LevelFilter};
use std::str::FromStr;

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_testBootstrapCore(
    env: JNIEnv,
    _: JClass,
    db_path_j_string: JString,
    level_j_string: JString,
    coepi_only: jboolean,
) -> jobject {
    let db_path_java_str = env.get_string(db_path_j_string).unwrap();
    let db_path_str = db_path_java_str.to_str().map_err(ServicesError::from);

    let level_java_str = env.get_string(level_j_string).unwrap();
    let level_str = level_java_str.to_str().unwrap();

    let coepi_only = coepi_only != 0;

    let filter_level = LevelFilter::from_str(&level_str).expect("Incorrect log level selected!");
    let _ = simple_logger::setup_logger(filter_level, coepi_only);

    println!("Bootstrapping with db path: {:?}", db_path_str);
    let result = db_path_str.and_then(|path| init_db(path).map_err(ServicesError::from));
    info!("Bootstrapping result: {:?}", result);

    jni_void_result(1, None, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_testReturnAnAlert(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let alert = create_test_alert("123", 234324);
    let jobject = alert_to_jobject(alert, &env);

    jni_obj_result(
        1,
        None,
        JObject::from(jobject),
        "org/coepi/android/api/JniOneAlertResult",
        "Lorg/coepi/android/api/JniAlert;",
        &env,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_testReturnMultipleAlerts(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let alert1 = create_test_alert("123", 131321);
    let alert2 = create_test_alert("343356", 32516899200);
    let jobject1 = alert_to_jobject(alert1, &env);
    let jobject2 = alert_to_jobject(alert2, &env);

    let array: jobjectArray = env
        .new_object_array(2, "org/coepi/android/api/JniAlert", jobject1)
        .unwrap();
    env.set_object_array_element(array, 0, jobject1).unwrap();
    env.set_object_array_element(array, 1, jobject2).unwrap();

    jni_obj_result(
        1,
        None,
        JObject::from(array),
        "org/coepi/android/api/JniAlertsArrayResult",
        "[Lorg/coepi/android/api/JniAlert;",
        &env,
    )
}

fn create_test_alert(id: &str, report_time: u64) -> Alert {
    let report = PublicReport {
        report_time: UnixTime { value: report_time },
        earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
        fever_severity: FeverSeverity::Mild,
        cough_severity: CoughSeverity::Dry,
        breathlessness: true,
        muscle_aches: true,
        loss_smell_or_taste: false,
        diarrhea: false,
        runny_nose: true,
        other: false,
        no_symptoms: true,
    };

    Alert {
        id: id.to_owned(),
        report,
        contact_time: 1592567315,
    }
}

fn alert_to_jobject(alert: Alert, env: &JNIEnv) -> jobject {
    let jni_public_report_class = env
        .find_class("org/coepi/android/api/JniPublicReport")
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

    let jni_alert_class = env.find_class("org/coepi/android/api/JniAlert").unwrap();

    let id_j_string = env.new_string(alert.id).unwrap();
    let id_j_value = JValue::from(JObject::from(id_j_string));

    let earliest_time_j_value = JValue::from(alert.contact_time as i64);

    env.new_object(
        jni_alert_class,
        "(Ljava/lang/String;Lorg/coepi/android/api/JniPublicReport;J)V",
        &[
            id_j_value,
            JValue::from(jni_public_report_obj.unwrap()),
            earliest_time_j_value,
        ],
    )
    .unwrap()
    .into_inner()
}
