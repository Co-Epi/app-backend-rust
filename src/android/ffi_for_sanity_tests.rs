use std::{
    sync::mpsc::{self, Sender},
    thread,
};
extern crate jni;
use self::jni::JNIEnv;
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::{jint, jobject, jstring};
use jni::JavaVM;
use log::warn;
use mpsc::Receiver;

#[derive(Debug)]
pub struct FFIParameterStruct {
    my_int: i32,
    my_str: String,
    my_nested: FFINestedParameterStruct,
}

#[derive(Debug)]
pub struct FFINestedParameterStruct {
    my_u8: u8,
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_sendReceiveString(
    env: JNIEnv,
    _: JClass,
    string: JString,
) -> jstring {
    let string: String = env
        .get_string(string)
        .expect("Couldn't create java string")
        .into();

    println!("Got string: {}", string);

    let output = env
        .new_string(format!("Hello {}!", string))
        .expect("Couldn't create java string");

    output.into_inner()
}

// #[no_mangle]
// pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_postReport(
//     env: JNIEnv,
//     _: JClass,
//     java_c_report: JString,
//     callback: JObject,
// ) -> jstring {
//     let str_parameter = env
//         .new_string("my string parameter")
//         .expect("Couldn't create java string!");

//     // Our Java companion code might pass-in "world" as a string, hence the name.
//     let world = rust_greeting2(
//         env.get_string(java_c_report)
//             .expect("invalid pattern string")
//             .as_ptr(),
//     );
//     // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
//     let world_ptr = CString::from_raw(world);
//     let output = env
//         .new_string(world_ptr.to_str().unwrap())
//         .expect("Couldn't create java string!");

//     let cls = env.find_class("org/coepi/android/api/FFIParameterStruct");
//     // let cls2 = env.find_class("org/coepi/android/api/Callback");

//     // let cls2_as_str = format!("{:?}", cls2.is_ok());

//     // let method_id = env.get_method_id(cls2.unwrap().clone(), "call", "(I)V");

//     // let res = format!("{:?}, {:?}", cls2_as_str, method_id.is_ok());

//     let number: jint = 123;
//     let numberAsJValue: JValue = number.into();
//     // env.call_method(callback, "call", "(I)V", &[numberAsJValue])
//     //     .unwrap();

//     let res: jint = 123;

//     let res = env.call_method(callback, "call", "(I)V", &[numberAsJValue]);
//     // .unwrap();

//     println!("res: {:?}", res);

//     // env.call_method(obj, name, sig, args)

//     // let output2 = env.new_string("???").expect("Couldn't create java string!");

//     // output2.into_inner()
//     str_parameter.into_inner()
// }

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_passStruct(
    env: JNIEnv,
    _: JClass,
    my_struct: JObject,
    callback: JObject,
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
            "Lorg/coepi/android/api/FFINestedParameterStruct;",
        )
        .unwrap();

    let my_nested_struct_j_value = my_nested_struct_j_value.l().unwrap();
    let my_nested_struct_my_u8_j_value = env
        .get_field(my_nested_struct_j_value, "myU8", "I")
        .unwrap();

    let my_nested_struct_my_u8 = my_nested_struct_my_u8_j_value.i().unwrap();

    let output2 = env
        .new_string(format!(
            "my_nested_struct_my_u8: {:?}",
            my_nested_struct_my_u8
        ))
        .expect("Couldn't create java string!");

    let a = JValue::from(JObject::from(output2));
    env.call_method(callback, "log", "(Ljava/lang/String;)V", &[a])
        .unwrap();

    let my_struct = FFIParameterStruct {
        my_int,
        my_str: my_str.to_owned(),
        my_nested: FFINestedParameterStruct {
            my_u8: my_nested_struct_my_u8 as u8,
        },
    };

    let output3 = env
        .new_string(format!("my_struct: {:?}", my_struct))
        .expect("Couldn't create java string!");

    let a2 = JValue::from(JObject::from(output3));
    env.call_method(callback, "log", "(Ljava/lang/String;)V", &[a2])
        .unwrap();

    1
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_returnStruct(
    env: JNIEnv,
    _: JClass,
    callback: JObject,
) -> jobject {
    let cls = env.find_class("org/coepi/android/api/FFIParameterStruct");

    let my_int = JValue::from(123);
    let str_parameter = env
        .new_string("my string parameter")
        .expect("Couldn't create java string!");

    let str_parameter_j_value = JValue::from(JObject::from(str_parameter));

    let nested = env.find_class("org/coepi/android/api/FFINestedParameterStruct");
    let my_int_nested = JValue::from(123);

    let nested_obj = env.new_object(nested.unwrap(), "(I)V", &[my_int_nested]);
    let nested_obj_val = JValue::from(nested_obj.unwrap());

    let obj = env.new_object(
        cls.unwrap(),
        "(ILjava/lang/String;Lorg/coepi/android/api/FFINestedParameterStruct;)V",
        &[my_int, str_parameter_j_value, nested_obj_val],
    );

    obj.unwrap().into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_callCallback(
    env: JNIEnv,
    _: JClass,
    callback: JObject,
) -> jint {
    let str = env.new_string("hi!").expect("Couldn't create java string!");

    let callback_arg = JValue::from(JObject::from(str));
    env.call_method(callback, "log", "(Ljava/lang/String;)V", &[callback_arg])
        .unwrap();
    1
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_registerCallback(
    env: JNIEnv,
    _: JClass,
    callback: jobject,
    // callback2: jobject,
) -> jint {
    let str = env.new_string("hi!").expect("Couldn't create java string!");

    // let callback_arg = JValue::from(JObject::from(str));
    // env.call_method(callback, "log", "(Ljava/lang/String;)V", &[callback_arg])
    //     .unwrap();

    // let foo = env.get_java_vm();

    // let mut java_vm: *mut JavaVM = ::std::ptr::null_mut();
    // let ret = unsafe { env.get_java_vm().unwrap()(env, &mut java_vm) };
    // assert_eq!(0, ret, "get_java_vm failed");

    // let global_callback_obj = unsafe { env.NewGlobalRef.unwrap()(env, callback) };
    // assert!(!global_callback_obj.is_null());

    // let foo: Result<JavaVM, jni::errors::Error> = env.get_java_vm();

    let my_callback = MyCallbackImpl {
        java_vm: env.get_java_vm().unwrap(),
        callback: env.new_global_ref(callback).unwrap(),
    };
    register_callback_internal(Box::new(my_callback));

    // let number: jint = 123;
    // let numberAsJValue: JValue = number.into();
    // let call_callback_res = my_callback
    //     .env
    //     .call_method(
    //         JObject::from(my_callback.callback),
    //         "call",
    //         "(I)V",
    //         &[numberAsJValue],
    //     )
    //     .unwrap();

    // let output = env
    //     .new_string(format!("call_callback_res: {:?}", call_callback_res))
    //     .expect("Couldn't create java string!");
    // let output_as_jvalue = JValue::from(JObject::from(output));
    // env.call_method(
    //     JObject::from(callback2),
    //     "log",
    //     "(Ljava/lang/String;)V",
    //     &[output_as_jvalue],
    // )
    // .unwrap();

    1
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_coepi_android_api_NativeApi_triggerCallback(
    env: JNIEnv,
    _: JClass,
    my_int: jint,
) -> jint {
    let my_int = my_int as i32;
    match &SENDER {
        // Push element to SENDER.
        Some(s) => {
            println!(">>> my_int: {}", my_int);
            s.send(my_int).expect("Couldn't send");
            1
        }

        None => {
            warn!("No callback registered");
            0
        }
    }
}

pub static mut SENDER: Option<Sender<i32>> = None;

trait MyCallback {
    fn call(&self, par: i32);
}

struct MyCallbackImpl {
    java_vm: JavaVM,
    callback: GlobalRef,
}

impl MyCallback for MyCallbackImpl {
    fn call(&self, par: i32) {
        let par_j_value = JValue::from(par);
        let env = self.java_vm.attach_current_thread().unwrap();
        // env.call_method(JObject::from(self.callback), "call", "(I)V", &[par_j_value]);
        env.call_method(self.callback.as_obj(), "call", "(I)V", &[par_j_value]);
    }
}

fn register_callback_internal(callback: Box<dyn MyCallback>) {
    // Make callback implement Send (marker for thread safe, basically) https://doc.rust-lang.org/std/marker/trait.Send.html
    let my_callback =
        unsafe { std::mem::transmute::<Box<dyn MyCallback>, Box<dyn MyCallback + Send>>(callback) };

    // Create channel
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

    // Save the sender in a static variable, which will be used to push elements to the callback
    unsafe {
        SENDER = Some(tx);
    }

    // Thread waits for elements pushed to SENDER and calls the callback
    thread::spawn(move || {
        for par in rx.iter() {
            println!(">>> receiver received par: {}", par);

            my_callback.call(par)
        }
    });
}
