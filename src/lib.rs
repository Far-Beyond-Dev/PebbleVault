#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)] // silence u128 being not FFI safe warnings.
                           // N.B. If any undefined behaviour occurs, it may be worthwhile to look
                           // into this FIRST.


mod ffi;
use ffi::*;

use MySQLGeo::Database;
mod MySQLGeo;

use std::ffi::{c_char, CStr, CString, c_void};

pub fn create_db() -> usize {
    let db = ffi::CreateDB();
    return db;
}

pub fn close_db(db: usize) {
    ffi::CloseDB(db);
    ffi::FreeDBPointer(db);
}

////////////////////////////////////////////////////////////////////
//  Lets define some functions for our API, these will allow the  //
//  user of the database to run queries against their vault       //
////////////////////////////////////////////////////////////////////
/* 
pub struct Vault {
    mem_db_handle: *mut c_void,
    sql_db_handle: MySQLGeo::Database,
}

impl Vault {
    pub fn new() -> Self {
        let mem_db_handle: *mut c_void = unsafe { CreateDB() as *mut c_void };
        println!("Memory DB Created");
        println!("Memory DB Handle: {:?}", mem_db_handle);

        let sql_db_handle = MySQLGeo::Database::new("data").unwrap();
        MySQLGeo::Database::create_table(&sql_db_handle);

        Vault {
            mem_db_handle,
            sql_db_handle,
        }
    }

    pub fn collect(&self, key: &str, data: &str) {
        println!("Collecting pebble: {}", key);
        // Implement the logic to store the data in memory and possibly MySQL
    }

    pub fn throw(&self, key: &str) {
        println!("Throwing pebble: {}", key);
        // Implement the logic to persist the data in MySQL
    }

    pub fn drop(&self, key: &str) {
        println!("Dropping pebble: {}", key);
        // Implement the logic to remove the data from memory and MySQL
    }

    pub fn skim(&self, key: &str) -> Option<String> {
        println!("Skimming pebble: {}", key);
        // Implement the logic to retrieve the data from memory or MySQL
        None
    }

    pub fn pebblestack(&self, table_name: &str) {
        println!("Creating pebblestack: {}", table_name);
        // Implement the logic to create a new table or collection
    }

    pub fn pebbledump(&self, table_name: &str, data: Vec<&str>) {
        println!("Dumping pebbles into stack: {}", table_name);
        // Implement the logic to bulk insert data into the table or collection
    }

    pub fn pebbleshift(&self, key: &str, new_data: &str) {
        println!("Shifting pebble: {}", key);
        // Implement the logic to update the data of an existing entry
    }

    pub fn pebblesift(&self, table_name: &str, query_conditions: &str) -> Vec<String> {
        println!("Sifting pebbles in stack: {}", table_name);
        // Implement the logic to query and filter data from the table or collection
        vec![]
    }

    pub fn pebblepatch(&self, key: &str, partial_data: &str) {
        println!("Patching pebble: {}", key);
        // Implement the logic to partially update the data of an existing entry
    }

    pub fn pebbleflow<F>(&self, transaction: F)
    where
        F: FnOnce(&Transaction),
    {
        println!("Starting pebbleflow transaction");
        let txn = Transaction::new();
        transaction(&txn);
        // Implement the logic to execute the transaction atomically
    }

    pub fn pebblesquash(&self, table_name: &str) {
        println!("Squashing pebblestack: {}", table_name);
        // Implement the logic to delete a table or collection
    }
}

pub struct Transaction {
    // Transaction implementation details
}

impl Transaction {
    pub fn new() -> Self {
        Transaction {
            // Initialize transaction details
        }
    }

    pub fn collect(&self, key: &str, data: &str) {
        println!("Transaction collect: {}", key);
        // Implement transaction logic for collect
    }

    pub fn throw(&self, key: &str) {
        println!("Transaction throw: {}", key);
        // Implement transaction logic for throw
    }

    pub fn drop(&self, key: &str) {
        println!("Transaction drop: {}", key);
        // Implement transaction logic for drop
    }
}
*/