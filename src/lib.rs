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

pub fn close_db(db: *mut c_void ) {
    unsafe {
        CloseDB(db as usize);
    }
}

pub fn create_spatial_index(db: *mut c_void, index_name: &str, index_key: &str) {
    let index_name = CString::new(index_name).unwrap();
    let index_key = CString::new(index_key).unwrap();

    println!("Creating Spatial Index: {} {}", index_name.to_str().unwrap(), index_key.to_str().unwrap());

    unsafe {
        CreateSpatialIndex(db as usize, index_name.as_ptr() as *mut c_char, index_key.as_ptr() as *mut c_char);
    }
}

pub fn create_galaxy(db: *mut c_void, key: &str, value: &str) {
    let key = CString::new(key).unwrap();
    let value = CString::new(value).unwrap();

    println!("Creating Galaxy: {} {}", key.to_str().unwrap(), value.to_str().unwrap());

    unsafe {
        CreateGalaxy(db as usize, key.as_ptr() as *mut c_char, value.as_ptr() as *mut c_char);
    }
}

pub fn get_k_nearest_galaxies(db: *mut c_void, key: &str) -> String {
    let key = CString::new(key).unwrap();
    
    print!("Getting K Nearest Galaxies: {}", key.to_str().unwrap());

    unsafe {
        let cstr = CStr::from_ptr(GetKNearestGalaxys(db as usize, key.as_ptr() as *mut c_char));
        let s = String::from_utf8_lossy(cstr.to_bytes()).to_string();
        GoFree(cstr.as_ptr() as *mut c_char);
        s
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
        let db = create_db();
        println!("Result: {}", db);
        assert_eq!(db, 1);

        db
    }

    #[test]
    fn test_close_db(db: *mut c_void) {
        close_db(db);
    }

    #[test]
    fn test_create_spatial_index() {
        create_spatial_index(db, "fleet", "fleet:*:pos");
    }

    #[test]
    fn test_create_galaxy() {
        create_galaxy(db, "galaxy:1", "data");
    }

    #[test]
    fn test_get_k_nearest_galaxies() {
        let result = get_k_nearest_galaxies(db, "galaxy:1");
        println!("Result: {}", result);
    }
}