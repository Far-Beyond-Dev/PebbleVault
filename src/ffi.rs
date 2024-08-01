///////////////////////////////////////////////////////////////////////////////////////
// This is where the FFIs for GoLang should be migrated, we will then use lib.rs to  //
// handel the public funstion calls which will each call into this file and/or the   //
// MySQLGeo.rs file                                                                  //
///////////////////////////////////////////////////////////////////////////////////////

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)] // silence u128 being not FFI safe warnings.
                           // N.B. If any undefined behaviour occurs, it may be worthwhile to look
                           // into this FIRST.

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use serde_json::Value;

use std::ffi::{c_char, CStr, CString, c_void};
use libc;

pub fn CreateDB() -> usize {
  unsafe {
      create_in_memory_db() as usize
  }
}

pub fn CloseDB(db: usize) {
  unsafe {
      close_in_memory_db(db as usize);
  }
}

pub fn FreeDBPointer(db: usize) {
  unsafe {
      free_in_memory_pointer_db(db as usize);
  }
}

pub fn SetObject(db: usize, key: &str, value: &str) {
  let c_key = CString::new(key).unwrap();
  let c_value = CString::new(value).unwrap();
  unsafe {
      set_object(db as usize, c_key.as_ptr() as *mut c_char, c_value.as_ptr() as *mut c_char);
  }
}

pub fn GetObject(db: usize, key: &str) -> Option<String> {
  let c_key = CString::new(key).unwrap();
  unsafe {
      let result = get_object(db as usize, c_key.as_ptr() as *mut c_char);
      if result.is_null() {
          None
      } else {
          let c_str = CStr::from_ptr(result);
          let string = c_str.to_string_lossy().into_owned();
          libc::free(result as *mut libc::c_void);
          Some(string)
      }
  }
}

pub fn DeleteObject(db: usize, key: &str) {
  let c_key = CString::new(key).unwrap();
  unsafe {
      delete_object(db as usize, c_key.as_ptr() as *mut c_char);
  }
}

pub fn GetAllObjects(db: usize) -> Option<String> {
  unsafe {
      let result = get_all_objects(db as usize);
      if result.is_null() {
          None
      } else {
          let c_str = CStr::from_ptr(result);
          let string = c_str.to_string_lossy().into_owned();
          libc::free(result as *mut libc::c_void);
          Some(string)
      }
  }
}

pub fn SetCustomIndexObjects(db: usize, index_name: &str, index_key: &str) {
  let c_index_name = CString::new(index_name).unwrap();
  let c_index_key = CString::new(index_key).unwrap();
  unsafe {
      set_custom_index_objects(db as usize, c_index_name.as_ptr() as *mut c_char, c_index_key.as_ptr() as *mut c_char);
  }
}

pub fn AddObjectToCustomIndex(db: usize, key: &str, value: &str) {
  let c_key = CString::new(key).unwrap();
  let c_value = CString::new(value).unwrap();
  unsafe {
      add_object_to_custom_index(db as usize, c_key.as_ptr() as *mut c_char, c_value.as_ptr() as *mut c_char);
  }
}

pub fn IterateOverCustomIndex(db: usize, index_name: &str) -> Option<String> {
  let c_index_name = CString::new(index_name).unwrap();
  unsafe {
      let result = iterate_over_custom_index(db as usize, c_index_name.as_ptr() as *mut c_char);
      if result.is_null() {
          None
      } else {
          let c_str = CStr::from_ptr(result);
          let string = c_str.to_string_lossy().into_owned();
          libc::free(result as *mut libc::c_void);
          Some(string)
      }
  }
}

pub fn CreateSpatialIndex(db: usize, index_name: &str) {
    let c_index_name = CString::new(index_name).unwrap();
    unsafe {
        create_spatial_index(db as usize, c_index_name.as_ptr() as *mut c_char);
    }
}

pub fn AddObjectToSpatialIndex(db: usize, json_data: &str) {
    let c_json_data = CString::new(json_data).unwrap();
    unsafe {
        add_object_to_spatial_index(db as usize, c_json_data.as_ptr() as *mut c_char);
    }
}

pub fn QuerySpatialIndexByArea(db: usize, index_name: &str, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Option<String> {
    let c_index_name = CString::new(index_name).unwrap();
    unsafe {
        let result = query_spatial_index_by_area(db as usize, c_index_name.as_ptr() as *mut c_char, min_x, min_y, min_z, max_x, max_y, max_z);
        if result.is_null() {
            println!("FFI: Null result from query_spatial_index_by_area");
            None
        } else {
            let c_str = CStr::from_ptr(result);
            println!("FFI: Raw C string result: {:?}", c_str);
            let string = c_str.to_string_lossy().into_owned();
            libc::free(result as *mut libc::c_void);
            if string == "[]" {
                println!("FFI: Empty result set");
                None
            } else {
                Some(string)
            }
        }
    }
}

pub fn Index3D(s: &str) -> Option<([f64; 3], [f64; 3])> {
    if let Ok(obj) = serde_json::from_str::<Value>(s) {
        let x = obj["x"].as_f64()?;
        let y = obj["y"].as_f64()?;
        let z = obj["z"].as_f64()?;
        Some(([x, y, z], [x, y, z]))
    } else {
        None
    }
}

pub fn QueryObjectByUUID(db: usize, uuid: &str) -> Option<String> {
  let c_uuid = CString::new(uuid).unwrap();
  unsafe {
      let result = query_object_by_uuid(db as usize, c_uuid.as_ptr() as *mut c_char);
      if result.is_null() {
          None
      } else {
          let c_str = CStr::from_ptr(result);
          let string = c_str.to_string_lossy().into_owned();
          libc::free(result as *mut libc::c_void);
          Some(string)
      }
  }
}

pub fn DeleteObjectByUUID(db: usize, uuid: &str) {
  let c_uuid = CString::new(uuid).unwrap();
  unsafe {
      delete_object_by_uuid(db as usize, c_uuid.as_ptr() as *mut c_char);
  }
}

pub fn UpdateObjectByUUID(db: usize, uuid: &str, json_data: &str) {
  let c_uuid = CString::new(uuid).unwrap();
  let c_json_data = CString::new(json_data).unwrap();
  unsafe {
      update_object_by_uuid(db as usize, c_uuid.as_ptr() as *mut c_char, c_json_data.as_ptr() as *mut c_char);
  }
}
/* 
pub fn QueryObjectsByType(db: usize, object_type: &str) -> Option<String> {
  let c_object_type = CString::new(object_type).unwrap();
  unsafe {
      let result = query_objects_by_type(db as usize, c_object_type.as_ptr() as *mut c_char);
      if result.is_null() {
          None
      } else {
          let c_str = CStr::from_ptr(result);
          let string = c_str.to_string_lossy().into_owned();
          libc::free(result as *mut libc::c_void);
          Some(string)
      }
  }
}

pub fn DeleteObjectsByType(db: usize, object_type: &str) {
  let c_object_type = CString::new(object_type).unwrap();
  unsafe {
      delete_objects_by_type(db as usize, c_object_type.as_ptr() as *mut c_char);
  }
}

pub fn QueryObjectsByTypeAndArea(db: usize, object_type: &str, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Option<String> {
  let c_object_type = CString::new(object_type).unwrap();
  unsafe {
      let result = query_objects_by_type_and_area(db as usize, c_object_type.as_ptr() as *mut c_char, min_x, min_y, min_z, max_x, max_y, max_z);
      if result.is_null() {
          None
      } else {
          let c_str = CStr::from_ptr(result);
          let string = c_str.to_string_lossy().into_owned();
          libc::free(result as *mut libc::c_void);
          Some(string)
      }
  }
}
*/



pub fn main() {
  // Create a new in-memory database
  let db = CreateDB();
  println!("Created in-memory database: {}", db);

  // Set an object
  SetObject(db, "key1", r#"{"type": "car", "uuid": "abc-123", "x": 1.0, "y": 2.0, "z": 3.0}"#);
  println!("Set object with key 'key1'");

  // Get the object
  if let Some(value) = GetObject(db, "key1") {
      println!("Retrieved object: {}", value);
  } else {
      println!("Failed to retrieve object");
  }

  // Create a spatial index
    CreateSpatialIndex(db, "spatial_index");
    println!("Created spatial index");

    // Add object to spatial index
    AddObjectToSpatialIndex(db, r#"{"type": "car", "uuid": "abc-123", "x": 1.0, "y": 2.0, "z": 3.0}"#);
    println!("Added object to spatial index");

  // Query spatial index by area
  if let Some(result) = QuerySpatialIndexByArea(db, "spatial_index", 0.0, 0.0, 0.0, 2.0, 3.0, 4.0) {
      println!("Queried spatial index: {}", result);
  } else {
      println!("No results from spatial index query");
  }

  // Query object by UUID
  if let Some(result) = QueryObjectByUUID(db, "abc-123") {
      println!("Queried object by UUID: {}", result);
  } else {
      println!("No object found with UUID");
  }

  // Update object by UUID
  UpdateObjectByUUID(db, "abc-123", r#"{"type": "car", "uuid": "abc-123", "x": 1.5, "y": 2.5, "z": 3.5}"#);
  println!("Updated object by UUID");

/* 
  // Query objects by type
  if let Some(result) = QueryObjectsByType(db, "car") {
      println!("Queried objects by type: {}", result);
  } else {
      println!("No objects found of type 'car'");
  }

  // Query objects by type and area
  if let Some(result) = QueryObjectsByTypeAndArea(db, "car", 0.0, 0.0, 0.0, 2.0, 3.0, 4.0) {
      println!("Queried objects by type and area: {}", result);
  } else {
      println!("No objects found of type 'car' in specified area");
  }
*/
  // Delete object by UUID
  DeleteObjectByUUID(db, "abc-123");
  println!("Deleted object by UUID");

  // Verify deletion
  if let Some(_) = QueryObjectByUUID(db, "abc-123") {
      println!("Object still exists (unexpected)");
  } else {
      println!("Object successfully deleted");
  }

  // Get all objects (should be empty now)
  if let Some(all_objects) = GetAllObjects(db) {
      println!("All objects: {}", all_objects);
  } else {
      println!("No objects in database");
  }

  // Close and free the database
  CloseDB(db);
  FreeDBPointer(db);
  println!("Closed and freed database");
}