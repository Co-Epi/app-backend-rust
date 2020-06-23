use std::{
    sync::mpsc::{self, Sender},
    thread,
};
extern crate jni;
use self::jni::JNIEnv;
use crate::expect_log;
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::{jfloat, jint, jobject, jstring};
use jni::JavaVM;
use log::error;
use log::{debug, warn};
use mpsc::Receiver;

#[derive(Debug)]
pub struct FFIParameterStruct {
    my_int: i32,
    my_str: String,
    my_nested: FFINestedParameterStruct,
}

#[derive(Debug)]
pub struct FFINestedParameterStruct {
    my_u8: i32,
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_sendReceiveString(
    env: JNIEnv,
    _: JClass,
    string: JString,
) -> jstring {
    let res = env.get_string(string);
    let string: String = expect_log!(res, "Couldn't create java string").into();

    let output_res = env.new_string(format!("Hello {}!", string));
    let output = expect_log!(output_res, "Couldn't create java string");

    output.into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_testPassAndReturnFloat(
    _env: JNIEnv,
    _: JClass,
    my_float: jfloat,
) -> jfloat {
    debug!("Passed jfloat: {}", my_float);
    let f = my_float as f32;
    debug!("f32: {}", f);
    f
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_passStruct(
    env: JNIEnv,
    _: JClass,
    my_struct: JObject,
) -> jint {
    let my_int_j_value_res = env.get_field(my_struct, "myInt", "I");
    let my_int: i32 = my_int_j_value_res.unwrap().i().unwrap();

    let my_str_j_value_res = env.get_field(my_struct, "myStr", "Ljava/lang/String;");
    let my_str_j_object = my_str_j_value_res.unwrap().l();
    let my_str_j_string = JString::from(my_str_j_object.unwrap());

    let my_str_java_string = env.get_string(my_str_j_string).unwrap();
    let my_str = my_str_java_string.to_str().unwrap();

    let my_nested_struct_j_value = env
        .get_field(
            my_struct,
            "myNested",
            "Lorg/coepi/android/core/FFINestedParameterStruct;",
        )
        .unwrap();

    let my_nested_struct_j_value = my_nested_struct_j_value.l().unwrap();
    let my_nested_struct_my_u8_j_value = env
        .get_field(my_nested_struct_j_value, "myU8", "I")
        .unwrap();

    let my_nested_struct_my_u8 = my_nested_struct_my_u8_j_value.i().unwrap();

    let _my_struct = FFIParameterStruct {
        my_int,
        my_str: my_str.to_owned(),
        my_nested: FFINestedParameterStruct {
            my_u8: my_nested_struct_my_u8 as i32,
        },
    };

    1
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_returnStruct(
    env: JNIEnv,
    _: JClass,
) -> jobject {
    let cls = env.find_class("org/coepi/android/core/FFIParameterStruct");

    let my_int_j_value = JValue::from(123);

    let str_parameter_res = env.new_string("my string parameter");
    let str_parameter = expect_log!(str_parameter_res, "Couldn't create java string!");
    let str_parameter_j_value = JValue::from(JObject::from(str_parameter));

    let nested = env.find_class("org/coepi/android/core/FFINestedParameterStruct");
    let my_int_nested = JValue::from(123);
    let nested_obj = env.new_object(nested.unwrap(), "(I)V", &[my_int_nested]);
    let nested_obj_val = JValue::from(nested_obj.unwrap());

    let obj = env.new_object(
        cls.unwrap(),
        "(ILjava/lang/String;Lorg/coepi/android/core/FFINestedParameterStruct;)V",
        &[my_int_j_value, str_parameter_j_value, nested_obj_val],
    );

    obj.unwrap().into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_callCallback(
    env: JNIEnv,
    _: JClass,
    callback: JObject,
) -> jint {
    let str_res = env.new_string("hi!");
    let str = expect_log!(str_res, "Couldn't create java string!");

    let callback_arg = JValue::from(JObject::from(str));
    env.call_method(callback, "call", "(Ljava/lang/String;)V", &[callback_arg])
        .unwrap();
    1
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_registerCallback(
    env: JNIEnv,
    _: JClass,
    callback: jobject,
) -> jint {
    let my_callback = MyCallbackImpl {
        java_vm: env.get_java_vm().unwrap(),
        callback: env.new_global_ref(callback).unwrap(),
    };
    register_callback_internal(Box::new(my_callback));
    1
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_core_NativeCore_triggerCallback(
    env: JNIEnv,
    _: JClass,
    string: JString,
) -> jint {
    let string_res = env.get_string(string);
    let string = expect_log!(string_res, "Couldn't create java string").into();

    match &SENDER {
        // Push element to SENDER.
        Some(s) => {
            let res = s.send(string);
            expect_log!(res, "Couldn't send");
            1
        }

        None => {
            warn!("No callback registered");
            0
        }
    }
}

pub static mut SENDER: Option<Sender<String>> = None;

trait MyCallback {
    fn call(&self, par: String);
}

struct MyCallbackImpl {
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

impl MyCallback for MyCallbackImpl {
    fn call(&self, par: String) {
        let env = self.java_vm.attach_current_thread().unwrap();

        let str_res = env.new_string(par);
        let str = expect_log!(str_res, "Couldn't create java string!");
        let str_j_value = JValue::from(JObject::from(str));

        let res = env.call_method(
            self.callback.as_obj(),
            "call",
            "(Ljava/lang/String;)V",
            &[str_j_value],
        );
        expect_log!(res, "Couldn't call callback");
    }
}

fn register_callback_internal(callback: Box<dyn MyCallback>) {
    // Make callback implement Send (marker for thread safe, basically) https://doc.rust-lang.org/std/marker/trait.Send.html
    let my_callback =
        unsafe { std::mem::transmute::<Box<dyn MyCallback>, Box<dyn MyCallback + Send>>(callback) };

    // Create channel
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // Save the sender in a static variable, which will be used to push elements to the callback
    unsafe {
        SENDER = Some(tx);
    }

    // Thread waits for elements pushed to SENDER and calls the callback
    thread::spawn(move || {
        for string in rx.iter() {
            my_callback.call(format!("{} world!", string))
        }
    });
}
