use std::ffi::CStr;
use std::os::raw::c_char;

pub fn retain_available_names(names: &mut Vec<*const c_char>, available_properties: &Vec<*const c_char>) {
    let available_names_cstr: Vec<_> = available_properties
        .iter()
        .map(|prop| unsafe { CStr::from_ptr(*prop) })
        .collect();

    names.retain(|&name| {
        let name_cstr = unsafe { CStr::from_ptr(name) };
        available_names_cstr.iter().any(|&available_name| {
            name_cstr == available_name
        })
    });
}

pub fn compare_c_str_value(value1: &*const c_char, value2: &*const c_char) -> bool {
    let value1_cstr = unsafe { CStr::from_ptr(*value1) };
    let value2_cstr = unsafe { CStr::from_ptr(*value2) };
    value1_cstr == value2_cstr
}

