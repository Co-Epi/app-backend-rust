use crate::{errors::ServicesError, init_db, simple_logger};
use jni::{
    objects::{GlobalRef, JClass, JObject, JString, JValue},
    sys::{jboolean, jobject},
    JNIEnv, JavaVM,
};
use log::{info, LevelFilter};
use mpsc::Receiver;
use simple_logger::{CoreLogLevel, CoreLogMessageThreadSafe, SENDER};
use std::str::FromStr;
use std::{
    sync::mpsc::{self, Sender},
    thread,
};

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_bootstrapCore(
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
    let db_result = db_path_str.and_then(|path| init_db(path).map_err(ServicesError::from));
    info!("Bootstrapping result: {:?}", db_result);

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
    let filter_level = LevelFilter::from_str(&level_str).expect("Incorrect log level selected!");
    let _ = simple_logger::setup_logger(filter_level, coepi_only != 0);
    log::max_level() as i32
}

pub fn jni_void_result(status: i32, message: Option<&str>, env: &JNIEnv) -> jobject {
    let cls = env.find_class("org/coepi/android/api/JniVoidResult");

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

        let text_j_string = env.new_string(text).expect("Couldn't create java string!");
        let text_j_value = JValue::from(JObject::from(text_j_string));

        env.call_method(
            self.callback.as_obj(),
            "log",
            "(ILjava/lang/String;)V",
            &[level_j_value, text_j_value],
        )
        .expect("Couldn't call callback");
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
