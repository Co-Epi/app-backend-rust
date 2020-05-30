
use super::ios_interface::cstring_to_str;
use core_foundation::{base::TCFType, string::{CFString, CFStringRef}};
use libc::c_char;

// Expose an interface for app's (for now only iOS) to test that general FFI is working as expected.
// i.e. assumptions on which the actual FFI interface relies.
// TODO can the c headers for this be generated in a separate file? Needs adjustments in script to generate lib and framework too.

#[repr(C)]
pub struct FFIParameterStruct {
  my_int: i32,
  my_str: *const c_char, // TODO use CFStringRef here too?
  my_nested: FFINestedParameterStruct,
}

#[repr(C)]
pub struct FFINestedParameterStruct {
  my_u8: u8,
}

#[derive(Debug)]
struct MyStruct {
  my_int: i32,
  my_str: String,
  my_u8: u8,
}

#[no_mangle]
pub unsafe extern "C" fn pass_struct(par: *const FFIParameterStruct) -> i32 {
  let my_str = cstring_to_str(&(*par).my_str).unwrap();

  let my_struct = MyStruct {
    my_int: (*par).my_int,
    my_str: my_str.to_owned(),
    my_u8: (*par).my_nested.my_u8,
  };

  println!("Received struct from iOS: {:?}", my_struct);

  1
}

#[repr(C)]
pub struct FFIReturnStruct {
  my_int: i32,
  my_str: CFStringRef,
  my_nested: FFINestedReturnStruct,
}

#[repr(C)]
pub struct FFINestedReturnStruct {
  my_u8: u8,
}

#[no_mangle]
pub unsafe extern "C" fn return_struct() -> FFIReturnStruct {
  let my_str = "hi!";
  let cf_string = CFString::new(&my_str.to_owned());
  let cf_string_ref = cf_string.as_concrete_TypeRef();

  ::std::mem::forget(cf_string);

  FFIReturnStruct {
    my_int: 123,
    my_str: cf_string_ref,
    my_nested: FFINestedReturnStruct { 
      my_u8: 255,
    },
  }
}

#[no_mangle]
pub unsafe extern "C" fn pass_and_return_struct(par: *const FFIParameterStruct) -> FFIReturnStruct {
  let my_str = cstring_to_str(&(*par).my_str).unwrap();
  let cf_string = CFString::new(&my_str.to_owned());
  let cf_string_ref = cf_string.as_concrete_TypeRef();

  ::std::mem::forget(cf_string);

  // TODO use CFStringRef in par?

  FFIReturnStruct {
    my_int: (*par).my_int,
    my_str: cf_string_ref,
    my_nested: FFINestedReturnStruct { 
      my_u8: (*par).my_nested.my_u8,
    },
  }
}