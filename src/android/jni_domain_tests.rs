use super::android_interface::{alert_to_jobject, jni_obj_result};
use crate::{
    reporting::{
        public_report::{CoughSeverity, FeverSeverity, PublicReport},
        symptom_inputs::UserInput,
    },
    reports_interval::UnixTime,
    reports_updater::Alert,
};
use jni::{
    objects::{JClass, JObject},
    sys::{jobject, jobjectArray},
    JNIEnv,
};

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_testReturnAnAlert(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let alert = create_test_alert("123", 234324);
    let jobject = alert_to_jobject(alert, &env);

    jni_obj_result(
        1,
        None,
        JObject::from(jobject),
        "org/coepi/android/core/JniOneAlertResult",
        "Lorg/coepi/android/core/JniAlert;",
        &env,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_testReturnMultipleAlerts(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let alert1 = create_test_alert("123", 131321);
    let alert2 = create_test_alert("343356", 32516899200);
    let jobject1 = alert_to_jobject(alert1, &env);
    let jobject2 = alert_to_jobject(alert2, &env);

    let array: jobjectArray = env
        .new_object_array(2, "org/coepi/android/core/JniAlert", jobject1)
        .unwrap();
    env.set_object_array_element(array, 0, jobject1).unwrap();
    env.set_object_array_element(array, 1, jobject2).unwrap();

    jni_obj_result(
        1,
        None,
        JObject::from(array),
        "org/coepi/android/core/JniAlertsArrayResult",
        "[Lorg/coepi/android/core/JniAlert;",
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
