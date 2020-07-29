use super::android_interface::{alert_to_jobject, jni_obj_result};
use crate::{
    expect_log,
    reporting::{
        public_symptoms::{CoughSeverity, FeverSeverity, PublicSymptoms},
        symptom_inputs::UserInput,
    },
    reports_interval::UnixTime,
    reports_update::reports_updater::Alert,
};
use jni::{
    objects::{JClass, JObject},
    sys::jobject,
    JNIEnv,
};
use log::error;

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_testReturnAnAlert(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let alert = create_test_alert("123", 234324);
    let res = alert_to_jobject(alert, &env);
    let jobject = expect_log!(res, "Failed creating alert jobject");

    jni_obj_result(
        1,
        None,
        JObject::from(jobject),
        "org/coepi/core/jni/JniOneAlertResult",
        "Lorg/coepi/core/jni/JniAlert;",
        &env,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_testReturnMultipleAlerts(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let alert1 = create_test_alert("123", 131321);
    let alert2 = create_test_alert("343356", 32516899200);

    let jobject1_res = alert_to_jobject(alert1, &env);
    let jobject2_res = alert_to_jobject(alert2, &env);
    let jobject1 = expect_log!(jobject1_res, "Couldn't create alert object");
    let jobject2 = expect_log!(jobject2_res, "Couldn't create alert object");

    let array_res = env.new_object_array(2, "org/coepi/core/jni/JniAlert", jobject1);
    let array = expect_log!(array_res, "Failed creating array jobject");

    env.set_object_array_element(array, 0, jobject1).unwrap();
    env.set_object_array_element(array, 1, jobject2).unwrap();

    jni_obj_result(
        1,
        None,
        JObject::from(array),
        "org/coepi/core/jni/JniAlertsArrayResult",
        "[Lorg/coepi/core/jni/JniAlert;",
        &env,
    )
}

fn create_test_alert(id: &str, report_time: u64) -> Alert {
    let symptoms = PublicSymptoms {
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
        report_id: id.to_owned(), // re-use alert id, not particular reason other than we don't need separate id for now
        symptoms,
        contact_start: 1592567315,
        contact_end: 1592567335,
        min_distance: 1.2,
        avg_distance: 2.1,
        is_read: false,
    }
}
