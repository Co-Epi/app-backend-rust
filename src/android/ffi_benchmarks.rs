extern crate jni;
use self::jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jobject, jstring};

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_noopForBenchmarks(env: JNIEnv, _: JClass) {}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_sendReceiveIntForBenchmarks(
    env: JNIEnv,
    _: JClass,
    i: jint,
) -> jint {
    1
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_sendCreateStringForBenchmarks(
    env: JNIEnv,
    _: JClass,
    string: JString,
) -> jstring {
    let string: String = env
        .get_string(string)
        .expect("Couldn't create java string")
        .into();

    let output = env
        .new_string("Return string")
        .expect("Couldn't create java string");

    output.into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_sendCreateStringDontUseInputForBenchmarks(
    env: JNIEnv,
    _: JClass,
    string: JString,
) -> jstring {
    let output = env
        .new_string("Return string")
        .expect("Couldn't create java string");

    output.into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_sendClassForBenchmarks(
    env: JNIEnv,
    _: JClass,
    my_struct: JObject,
) {
    let my_int_j_value_res = env.get_field(my_struct, "myInt", "I");
    let my_int: i32 = my_int_j_value_res.unwrap().i().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_core_jni_JniApi_returnClassForBenchmarks(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let cls = env.find_class("org/coepi/core/jni/BenchmarksIntClass");
    let my_int_j_value = JValue::from(123);
    let obj = env.new_object(cls.unwrap(), "(I)V", &[my_int_j_value]);
    obj.unwrap().into_inner()
}
