#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)] // silence u128 being not FFI safe warnings.
                           // N.B. If any undefined behaviour occurs, it may be worthwhile to look
                           // into this FIRST.

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::{
    c_char, 
    CStr, 
    CString, 
    c_void
};

pub fn greet(name: &str) -> String {
    let name = CString::new(name).unwrap();

    unsafe {
        let cstr = CStr::from_ptr(Greet(name.as_ptr() as *mut c_char));
        let s = String::from_utf8_lossy(cstr.to_bytes()).to_string();
        GoFree(cstr.as_ptr() as *mut c_char);
        s
    }
}

// in Go, the return type is uintptr, which is an unsigned integer type that is large enough to hold the bit pattern of any pointer.
// In Rust, we use *mut c_void to represent this type. its a opaque pointer.
pub fn create_db() -> *mut c_void {
    let db_handle = unsafe { CreateDB() as *mut c_void};
    println!("DB Created");
    println!("DB Handle: {:?}", db_handle as *mut c_void);
    db_handle as *mut c_void
}

pub fn close_db(db: usize) {
    unsafe {
        CloseDB(db as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = greet("Rust");
        assert_eq!(result, "Not Hello from Go, Rust!");
    }

    #[test]
    fn test_create_db() {
        let result = create_db();
        println!("Result: {}", result);
        assert_eq!(result, 1);

        result
    }

    #[test]
    fn test_close_db() {
        let db = create_db();
        close_db(db);
    }
}