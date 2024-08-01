#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

mod ffi;
mod MySQLGeo;

use std::collections::HashMap;
use std::ffi::CString;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct DbHandles {
    mem_db_handle: usize,
    //sql_db_handle: MySQLGeo::Database,
}

pub struct VaultManager {
    dbs: Arc<Mutex<HashMap<String, DbHandles>>>,
}

impl VaultManager {
    pub fn new() -> Self {
        VaultManager {
            dbs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn obfuscate_name(&self, name: &str) -> String {
        // Simple obfuscation: reverse the string and add a prefix
        let mut obfuscated = name.chars().rev().collect::<String>();
        obfuscated.push_str("_obf");
        obfuscated
    }

    pub fn create_db(&self, name: &str) -> Result<(), String> {
        let obfuscated_name = self.obfuscate_name(name);
        let mut dbs = self.dbs.lock().unwrap();
        
        if dbs.contains_key(&obfuscated_name) {
            return Err(format!("Database '{}' already exists", name));
        }

        let mem_db_handle = ffi::CreateDB();
        println!("Memory DB Created for '{}'", name);
        println!("Memory DB Handle: {:?}", mem_db_handle);

        let sql_db_handle = MySQLGeo::Database::new(&format!("data_{}", name)).unwrap();
        MySQLGeo::Database::create_table(&sql_db_handle);

        dbs.insert(obfuscated_name, DbHandles {
            mem_db_handle,
            //sql_db_handle,
        });

        Ok(())
    }

    fn get_db_handles(&self, name: &str) -> Result<DbHandles, String> {
        let obfuscated_name = self.obfuscate_name(name);
        let dbs = self.dbs.lock().unwrap();
        dbs.get(&obfuscated_name)
            .cloned()
            .ok_or_else(|| format!("Database '{}' not found", name))
    }

    /// Collects a pebble (stores an object) in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to store the object in.
    /// * `key` - The key to associate with the object.
    /// * `data` - The data to store, as a JSON string.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn collect(&self, db_name: &str, key: &str, data: &str) -> Result<(), String> {
        println!("Collecting pebble: {} in DB: {}", key, db_name);
        let handles = self.get_db_handles(db_name)?;
        ffi::SetObject(handles.mem_db_handle, key, data);
        Ok(())
    }

    /// Throws a pebble (persists an object) from in-memory to MySQL storage.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the object.
    /// * `key` - The key of the object to persist.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn throw(&self, db_name: &str, key: &str) -> Result<(), String> {
        println!("Throwing pebble: {} in DB: {}", key, db_name);
        let handles = self.get_db_handles(db_name)?;
        if let Some(data) = ffi::GetObject(handles.mem_db_handle, key) {
            // Implement logic to persist data to MySQL
            println!("Persisting to MySQL: {} - {}", key, data);
        }
        Ok(())
    }

    /// Drops a pebble (deletes an object) from the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the object.
    /// * `key` - The key of the object to delete.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn drop(&self, db_name: &str, key: &str) -> Result<(), String> {
        println!("Dropping pebble: {} in DB: {}", key, db_name);
        let handles: DbHandles = self.get_db_handles(db_name)?;
        ffi::DeleteObject(handles.mem_db_handle, key);
        // Implement logic to remove from MySQL if necessary
        Ok(())
    }

    /// Skims a pebble (retrieves an object) from the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the object.
    /// * `key` - The key of the object to retrieve.
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>, String>` - The object data if found, None if not found, or an error message if failed.
    pub fn skim(&self, db_name: &str, key: &str) -> Result<Option<String>, String> {
        println!("Skimming pebble: {} in DB: {}", key, db_name);
        let handles = self.get_db_handles(db_name)?;
        Ok(ffi::GetObject(handles.mem_db_handle, key))
    }

    /// Creates a new pebblestack (table or collection) in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to create the pebblestack in.
    /// * `table_name` - The name of the pebblestack to create.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn pebblestack(&self, db_name: &str, table_name: &str) -> Result<(), String> {
        println!("Creating pebblestack: {} in DB: {}", table_name, db_name);
        let _handles = self.get_db_handles(db_name)?;
        // Implement logic to create a new table or collection
        Ok(())
    }

    /// Dumps multiple pebbles (inserts multiple objects) into a pebblestack.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the pebblestack.
    /// * `table_name` - The name of the pebblestack to insert into.
    /// * `data` - A vector of JSON strings representing the objects to insert.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn pebbledump(&self, db_name: &str, table_name: &str, data: Vec<&str>) -> Result<(), String> {
        println!("Dumping pebbles into stack: {} in DB: {}", table_name, db_name);
        let handles = self.get_db_handles(db_name)?;
        for (index, item) in data.iter().enumerate() {
            let key = format!("{}:{}", table_name, index);
            ffi::SetObject(handles.mem_db_handle, &key, item);
        }
        Ok(())
    }

    /// Shifts a pebble (updates an object) in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the object.
    /// * `key` - The key of the object to update.
    /// * `new_data` - The new data to replace the existing object, as a JSON string.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn pebbleshift(&self, db_name: &str, key: &str, new_data: &str) -> Result<(), String> {
        println!("Shifting pebble: {} in DB: {}", key, db_name);
        let handles = self.get_db_handles(db_name)?;
        ffi::UpdateObjectByUUID(handles.mem_db_handle, key, new_data);
        Ok(())
    }

    /// Sifts pebbles (queries objects) in a pebblestack based on given conditions.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the pebblestack.
    /// * `table_name` - The name of the pebblestack to query.
    /// * `query_conditions` - A JSON string representing the query conditions.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, String>` - A vector of matching objects if successful, or an error message if failed.
    pub fn pebblesift(&self, db_name: &str, table_name: &str, query_conditions: &str) -> Result<Vec<String>, String> {
        println!("Sifting pebbles in stack: {} in DB: {}", table_name, db_name);
        let _handles = self.get_db_handles(db_name)?;
        // Implement query logic based on conditions
        Ok(vec![])
    }

    /// Patches a pebble (partially updates an object) in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the object.
    /// * `key` - The key of the object to patch.
    /// * `partial_data` - A JSON string containing the fields to update.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn pebblepatch(&self, db_name: &str, key: &str, partial_data: &str) -> Result<(), String> {
        println!("Patching pebble: {} in DB: {}", key, db_name);
        let handles = self.get_db_handles(db_name)?;
        if let Some(existing_data) = ffi::GetObject(handles.mem_db_handle, key) {
            if let (Ok(mut existing_json), Ok(patch_json)) = (serde_json::from_str::<Value>(&existing_data), serde_json::from_str::<Value>(partial_data)) {
                if let Value::Object(patch_map) = patch_json {
                    for (k, v) in patch_map {
                        existing_json[k] = v;
                    }
                }
                if let Ok(updated_data) = serde_json::to_string(&existing_json) {
                    ffi::UpdateObjectByUUID(handles.mem_db_handle, key, &updated_data);
                }
            }
        }
        Ok(())
    }

    /// Executes a transaction (pebbleflow) on the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to perform the transaction on.
    /// * `transaction` - A closure representing the transaction operations.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if the transaction was successful, or an error message if failed.
    pub fn pebbleflow<F>(&self, db_name: &str, transaction: F) -> Result<(), String>
    where
        F: FnOnce(&Transaction) -> Result<(), String>,
    {
        println!("Starting pebbleflow transaction in DB: {}", db_name);
        let handles = self.get_db_handles(db_name)?;
        let txn = Transaction::new(&handles);
        transaction(&txn)
    }

    /// Squashes a pebblestack (deletes a table or collection) in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the pebblestack.
    /// * `table_name` - The name of the pebblestack to delete.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn pebblesquash(&self, db_name: &str, table_name: &str) -> Result<(), String> {
        println!("Squashing pebblestack: {} in DB: {}", table_name, db_name);
        let _handles = self.get_db_handles(db_name)?;
        // Implement logic to delete a table or collection
        Ok(())
    }

/// Creates a spatial index in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to create the spatial index in.
    /// * `index_name` - The name of the spatial index to create.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn create_spatial_index(&self, db_name: &str, index_name: &str) -> Result<(), String> {
        println!("Creating spatial index: {} in DB: {}", index_name, db_name);
        let handles = self.get_db_handles(db_name)?;
        ffi::CreateSpatialIndex(handles.mem_db_handle, index_name);
        Ok(())
    }

    /// Adds an object to a spatial index in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the spatial index.
    /// * `uuid` - The UUID of the object.
    /// * `x`, `y`, `z` - The coordinates of the object.
    /// * `data` - Additional data associated with the object.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn add_object_to_spatial_index(&self, db_name: &str, uuid: &str, x: f64, y: f64, z: f64, data: &str) -> Result<(), String> {
        println!("Adding object to spatial index in DB: {}", db_name);
        let handles = self.get_db_handles(db_name)?;
        let json_data = json!({
            "uuid": uuid,
            "x": x,
            "y": y,
            "z": z,
            "data": data
        }).to_string();
        println!("JSON data being added: {}", json_data);
        ffi::AddObjectToSpatialIndex(handles.mem_db_handle, &json_data);
        Ok(())
    }
    /// Queries a spatial index by area in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the spatial index.
    /// * `index_name` - The name of the spatial index to query.
    /// * `min_x`, `min_y`, `min_z` - Minimum coordinates of the bounding box.
    /// * `max_x`, `max_y`, `max_z` - Maximum coordinates of the bounding box.
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>, String>` - A JSON string of matching objects if successful, None if no matches, or an error message if failed.
    pub fn query_spatial_index_by_area(&self, db_name: &str, index_name: &str, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Result<Option<String>, String> {
        println!("Querying spatial index: {} in DB: {} by area", index_name, db_name);
        println!("Bounding box: [{} {} {}],[{} {} {}]", min_x, min_y, min_z, max_x, max_y, max_z);
        let handles = self.get_db_handles(db_name)?;
        match ffi::QuerySpatialIndexByArea(handles.mem_db_handle, index_name, min_x, min_y, min_z, max_x, max_y, max_z) {
            Some(result) if result != "[]" => Ok(Some(result)),
            Some(_) => Ok(None),
            None => Ok(None),
        }
    }
    /// Queries an object by its UUID in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the object.
    /// * `uuid` - The UUID of the object to query.
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>, String>` - The object data if found, None if not found, or an error message if failed.
    pub fn query_object_by_uuid(&self, db_name: &str, uuid: &str) -> Result<Option<String>, String> {
        println!("Querying object by UUID: {} in DB: {}", uuid, db_name);
        let handles = self.get_db_handles(db_name)?;
        Ok(ffi::QueryObjectByUUID(handles.mem_db_handle, uuid))
    }

    /// Deletes an object by its UUID from the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the object.
    /// * `uuid` - The UUID of the object to delete.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn delete_object_by_uuid(&self, db_name: &str, uuid: &str) -> Result<(), String> {
        println!("Deleting object by UUID: {} in DB: {}", uuid, db_name);
        let handles = self.get_db_handles(db_name)?;
        ffi::DeleteObjectByUUID(handles.mem_db_handle, uuid);
        Ok(())
    }

    /// Updates an object by its UUID in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the object.
    /// * `uuid` - The UUID of the object to update.
    /// * `json_data` - A JSON string containing the updated object data.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn update_object_by_uuid(&self, db_name: &str, uuid: &str, json_data: &str) -> Result<(), String> {
        println!("Updating object by UUID: {} in DB: {}", uuid, db_name);
        let handles = self.get_db_handles(db_name)?;
        ffi::UpdateObjectByUUID(handles.mem_db_handle, uuid, json_data);
        Ok(())
    }
/* 
    /// Queries objects by type in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to query.
    /// * `object_type` - The type of objects to query for.
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>, String>` - A JSON string of matching objects if successful, None if no matches, or an error message if failed.
    pub fn query_objects_by_type(&self, db_name: &str, object_type: &str) -> Result<Option<String>, String> {
        println!("Querying objects by type: {} in DB: {}", object_type, db_name);
        let handles = self.get_db_handles(db_name)?;
        Ok(ffi::QueryObjectsByType(handles.mem_db_handle, object_type))
    }

    /// Deletes objects by type from the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the objects.
    /// * `object_type` - The type of objects to delete.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn delete_objects_by_type(&self, db_name: &str, object_type: &str) -> Result<(), String> {
        println!("Deleting objects by type: {} in DB: {}", object_type, db_name);
        let handles = self.get_db_handles(db_name)?;
        ffi::DeleteObjectsByType(handles.mem_db_handle, object_type);
        Ok(())
    }

    /// Queries objects by type and area in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to query.
    /// * `object_type` - The type of objects to query for.
    /// * `min_x`, `min_y`, `min_z` - Minimum coordinates of the bounding box.
    /// * `max_x`, `max_y`, `max_z` - Maximum coordinates of the bounding box.
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>, String>` - A JSON string of matching objects if successful, None if no matches, or an error message if failed.
    pub fn query_objects_by_type_and_area(&self, db_name: &str, object_type: &str, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Result<Option<String>, String> {
        println!("Querying objects by type: {} and area in DB: {}", object_type, db_name);
        let handles = self.get_db_handles(db_name)?;
        Ok(ffi::QueryObjectsByTypeAndArea(handles.mem_db_handle, object_type, min_x, min_y, min_z, max_x, max_y, max_z))
    }
*/
    /// Sets a custom index for objects in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to create the index in.
    /// * `index_name` - The name of the custom index to create.
    /// * `index_key` - The key or pattern to use for indexing.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn set_custom_index_objects(&self, db_name: &str, index_name: &str, index_key: &str) -> Result<(), String> {
        println!("Setting custom index: {} with key: {} in DB: {}", index_name, index_key, db_name);
        let handles = self.get_db_handles(db_name)?;
        ffi::SetCustomIndexObjects(handles.mem_db_handle, index_name, index_key);
        Ok(())
    }

    /// Adds an object to a custom index in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the custom index.
    /// * `key` - The key of the object to add to the index.
    /// * `value` - The value of the object to add to the index.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok(()) if successful, or an error message if failed.
    pub fn add_object_to_custom_index(&self, db_name: &str, key: &str, value: &str) -> Result<(), String> {
        println!("Adding object to custom index in DB: {}", db_name);
        let handles = self.get_db_handles(db_name)?;
        ffi::AddObjectToCustomIndex(handles.mem_db_handle, key, value);
        Ok(())
    }

    /// Iterates over a custom index in the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database containing the custom index.
    /// * `index_name` - The name of the custom index to iterate over.
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>, String>` - A string containing the iterated objects if successful, None if the index is empty, or an error message if failed.
    pub fn iterate_over_custom_index(&self, db_name: &str, index_name: &str) -> Result<Option<String>, String> {
        println!("Iterating over custom index: {} in DB: {}", index_name, db_name);
        let handles = self.get_db_handles(db_name)?;
        Ok(ffi::IterateOverCustomIndex(handles.mem_db_handle, index_name))
    }

    /// Retrieves all objects from the specified database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to retrieve objects from.
    ///
    /// # Returns
    ///
    /// * `Result<String, String>` - A string containing all objects if successful, or an error message if failed.
    pub fn get_all_objects(&self, db_name: &str) -> Result<String, String> {
        println!("Retrieving all objects from DB: {}", db_name);
        let handles = self.get_db_handles(db_name)?;
        match ffi::GetAllObjects(handles.mem_db_handle) {
            Some(objects) => Ok(objects),
            None => Ok(String::new()),
        }
    }
}

pub struct Transaction<'a> {
    handles: &'a DbHandles,
}

impl<'a> Transaction<'a> {
    fn new(handles: &'a DbHandles) -> Self {
        Transaction { handles }
    }

    pub fn collect(&self, key: &str, data: &str) {
        println!("Transaction collect: {}", key);
        ffi::SetObject(self.handles.mem_db_handle, key, data);
    }

    pub fn throw(&self, key: &str) {
        println!("Transaction throw: {}", key);
        // Implement transaction logic for throw
    }

    pub fn drop(&self, key: &str) {
        println!("Transaction drop: {}", key);
        ffi::DeleteObject(self.handles.mem_db_handle, key);
    }
}

impl Drop for VaultManager {
    fn drop(&mut self) {
        let dbs = self.dbs.lock().unwrap();
        for (_, handles) in dbs.iter() {
            ffi::CloseDB(handles.mem_db_handle);
            ffi::FreeDBPointer(handles.mem_db_handle);
        }
        println!("Closed and freed all databases");
    }
}

// Main function with examples
pub fn main() {
    let vault_manager = VaultManager::new();

    // Create a database
    match vault_manager.create_db("spatial_db") {
        Ok(_) => println!("Created database 'spatial_db'"),
        Err(e) => {
            println!("Error creating database: {}", e);
            return;
        }
    }

    // Create a spatial index
    match vault_manager.create_spatial_index("spatial_db", "location_index") {
        Ok(_) => println!("Created spatial index 'location_index' in 'spatial_db'"),
        Err(e) => {
            println!("Error creating spatial index: {}", e);
            return;
        }
    }

    // Add objects to the spatial index
    let test_objects = vec![
        ("point1", 10.0, 20.0, 30.0, "Point A"),
        ("point2", 15.0, 25.0, 35.0, "Point B"),
        ("point3", 5.0, 15.0, 25.0, "Point C"),
    ];

    for (uuid, x, y, z, data) in test_objects {
        match vault_manager.add_object_to_spatial_index("spatial_db", uuid, x, y, z, data) {
            Ok(_) => println!("Added object {} to spatial index", uuid),
            Err(e) => {
                println!("Error adding object {} to spatial index: {}", uuid, e);
                return;
            }
        }
    }
    
    println!("Added all objects to spatial index in 'spatial_db'");

    // Query the spatial index
    let query_tests = vec![
        (0.0, 0.0, 0.0, 20.0, 30.0, 40.0, "should return all points"),
        (0.0, 0.0, 0.0, 10.0, 20.0, 30.0, "should return point1 and point3"),
        (11.0, 21.0, 31.0, 20.0, 30.0, 40.0, "should return point2"),
        (30.0, 40.0, 50.0, 40.0, 50.0, 60.0, "should return no points"),
    ];

    for (min_x, min_y, min_z, max_x, max_y, max_z, description) in query_tests {
        println!("\nTesting query that {}", description);
        match vault_manager.query_spatial_index_by_area("spatial_db", "location_index", min_x, min_y, min_z, max_x, max_y, max_z) {
            Ok(Some(results)) => {
                println!("Spatial query raw results: {}", results);
                match serde_json::from_str::<Vec<serde_json::Value>>(&results) {
                    Ok(objects) => {
                        if objects.is_empty() {
                            println!("No objects found in query results");
                        } else {
                            println!("Spatial query parsed results:");
                            for obj in objects {
                                println!("Object: {:?}", obj);
                            }
                        }
                    },
                    Err(e) => println!("Error parsing JSON results: {}", e),
                }
            },
            Ok(None) => println!("No results found in spatial query"),
            Err(e) => println!("Error during spatial query: {}", e),
        }
    }

    // Test Index3D function
    let test_json = r#"{"uuid": "test", "x": 1.0, "y": 2.0, "z": 3.0, "data": "Test Point"}"#;
    if let Some((min, max)) = ffi::Index3D(test_json) {
        println!("\nIndex3D test - Min: {:?}, Max: {:?}", min, max);
    } else {
        println!("\nIndex3D test failed");
    }

    // Additional test: Retrieve all objects
    println!("\nRetrieving all objects:");
    match vault_manager.get_all_objects("spatial_db") {
        Ok(objects) => {
            if objects.is_empty() {
                println!("No objects found in the database");
            } else {
                for obj in objects.split(',') {
                    if !obj.is_empty() {
                        println!("Object: {}", obj);
                    }
                }
            }
        },
        Err(e) => println!("Error retrieving all objects: {}", e),
    }

    println!("\nAll operations completed");
}