#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)] // silence u128 being not FFI safe warnings.
                           // N.B. If any undefined behaviour occurs, it may be worthwhile to look
                           // into this FIRST.
mod ffi;
use ffi::*;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::ffi::{c_char, CStr, CString, c_void};

use MySQLGeo::Database;
mod MySQLGeo;

lazy_static! {
    static ref DB_HANDLES: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
}


fn get_handle(name: &str) -> Option<usize> {
    let handles = DB_HANDLES.lock().unwrap();
    handles.get(name).cloned()
}


pub struct Vault {
    mem_db_handles: Arc<Mutex<HashMap<String, usize>>>,
    sql_db_handle: MySQLGeo::Database,
}

impl Vault {
    pub fn new() -> Self {
        Vault {
            mem_db_handles: Arc::new(Mutex::new(HashMap::new())),
            sql_db_handle: MySQLGeo::Database::new("data").unwrap(),
        }
    }

    pub fn create_db(&self, name: &str) -> usize {
        let handle = unsafe { create_in_memory_db() as usize };
        let mut handles = self.mem_db_handles.lock().unwrap();
        handles.insert(name.to_string(), handle);
        handle
    }

    fn get_handle(&self, name: &str) -> Option<usize> {
        let handles = self.mem_db_handles.lock().unwrap();
        handles.get(name).cloned()
    }

    pub fn close_db(&self, name: &str) {
        if let Some(handle) = self.get_handle(name) {
            unsafe {
                close_in_memory_db(handle);
            }
            let mut handles = self.mem_db_handles.lock().unwrap();
            handles.remove(name);
        }
    }

    pub fn free_db_pointer(&self, name: &str) {
        if let Some(handle) = self.get_handle(name) {
            unsafe {
                free_in_memory_pointer_db(handle);
            }
            let mut handles = self.mem_db_handles.lock().unwrap();
            handles.remove(name);
        }
    }

    pub fn set_object(&self, name: &str, key: &str, value: &str) {
        if let Some(handle) = self.get_handle(name) {
            let c_key = CString::new(key).unwrap();
            let c_value = CString::new(value).unwrap();
            unsafe {
                set_object(handle, c_key.as_ptr() as *mut c_char, c_value.as_ptr() as *mut c_char);
            }
        }
    }

    pub fn get_object(&self, name: &str, key: &str) -> Option<String> {
        self.get_handle(name).and_then(|handle| {
            let c_key = CString::new(key).unwrap();
            unsafe {
                let result = get_object(handle, c_key.as_ptr() as *mut c_char);
                if result.is_null() {
                    None
                } else {
                    let c_str = CStr::from_ptr(result);
                    let string = c_str.to_string_lossy().into_owned();
                    libc::free(result as *mut libc::c_void);
                    Some(string)
                }
            }
        })
    }

    pub fn delete_object(&self, name: &str, key: &str) {
        if let Some(handle) = self.get_handle(name) {
            let c_key = CString::new(key).unwrap();
            unsafe {
                delete_object(handle, c_key.as_ptr() as *mut c_char);
            }
        }
    }

    pub fn collect(&self, key: &str, data: &str) {
        println!("Collecting pebble: {}", key);
        // TODO: Implement the logic to store the data in memory and possibly MySQL
        // This might involve calling set_object on an in-memory database and also storing in SQL
    }

    pub fn throw(&self, key: &str) {
        println!("Throwing pebble: {}", key);
        // TODO: Implement the logic to persist the data in MySQL
        // This might involve retrieving data from in-memory DB and storing it in SQL
    }

    pub fn drop(&self, key: &str) {
        println!("Dropping pebble: {}", key);
        // TODO: Implement the logic to remove the data from memory and MySQL
        // This might involve deleting from both in-memory DB and SQL
    }

    pub fn skim(&self, key: &str) -> Option<String> {
        println!("Skimming pebble: {}", key);
        // TODO: Implement the logic to retrieve the data from memory or MySQL
        // This might involve checking in-memory DB first, then SQL if not found
        None
    }

    pub fn pebblestack(&self, table_name: &str) {
        println!("Creating pebblestack: {}", table_name);
        // TODO: Implement the logic to create a new table or collection
        // This might involve creating a new table in SQL
    }

    pub fn pebbledump(&self, table_name: &str, data: Vec<&str>) {
        println!("Dumping pebbles into stack: {}", table_name);
        // TODO: Implement the logic to bulk insert data into the table or collection
        // This might involve bulk inserting into SQL
    }

    pub fn pebbleshift(&self, key: &str, new_data: &str) {
        println!("Shifting pebble: {}", key);
        // TODO: Implement the logic to update the data of an existing entry
        // This might involve updating both in-memory DB and SQL
    }

    pub fn pebblesift(&self, table_name: &str, query_conditions: &str) -> Vec<String> {
        println!("Sifting pebbles in stack: {}", table_name);
        // TODO: Implement the logic to query and filter data from the table or collection
        // This might involve querying SQL with the given conditions
        vec![]
    }

    pub fn pebblepatch(&self, key: &str, partial_data: &str) {
        println!("Patching pebble: {}", key);
        // TODO: Implement the logic to partially update the data of an existing entry
        // This might involve partial updates in both in-memory DB and SQL
    }

    //  pub fn pebbleflow<F>(&self, transaction: F)
    //  where
    //      F: FnOnce(&Transaction),
    //  {
    //      println!("Starting pebbleflow transaction");
    //      let txn = Transaction::new();
    //      transaction(&txn);
    //      // TODO: Implement the logic to execute the transaction atomically
    //      // This might involve managing a transaction across both in-memory DB and SQL
    //  }

    pub fn pebblesquash(&self, table_name: &str) {
        println!("Squashing pebblestack: {}", table_name);
        // TODO: Implement the logic to delete a table or collection
        // This might involve dropping a table in SQL
    }
}