//! # Load Testing Module for PebbleVault
//!
//! This module provides functionality to perform comprehensive load tests on the PebbleVault system.
//! It simulates high-load scenarios by creating, adding, persisting, loading, deleting, and re-adding
//! objects across multiple regions, with a focus on custom data handling.
//!
//! ## Key Features
//!
//! - Multi-region stress testing
//! - Large-scale object creation and manipulation
//! - Performance measurement for various operations
//! - Verification of data consistency under load
//! - Simulation of complex game-world scenarios
//! - Custom data integrity testing
//!
//! ## Usage
//!
//! The main entry point for load testing is the `run_load_test` function, which takes
//! a VaultManager instance and test parameters as input.

use super::*;
use std::time::{Instant, Duration};
use rand::Rng;
use uuid::Uuid;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::fmt::Debug;
use rand::distributions::{Distribution, Standard};

/// Custom data structure for load testing
///
/// This struct represents a complex game object with various attributes,
/// used to simulate custom data in the load testing scenarios.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
struct LoadTestData {
    name: String,
    level: u32,
    health: f32,
    inventory: Vec<String>,
    is_active: bool,
}

impl LoadTestData {
    /// Creates a new LoadTestData instance with random values
    fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        LoadTestData {
            name: format!("Object_{}", rng.gen::<u32>()),
            level: rng.gen_range(1..100),
            health: rng.gen_range(0.0..100.0),
            inventory: (0..rng.gen_range(0..5))
                .map(|_| format!("Item_{}", rng.gen::<u32>()))
                .collect(),
            is_active: rng.gen_bool(0.5),
        }
    }
}

/// Formats a Duration into a string with seconds and microseconds.
///
/// This helper function is used to present timing information in a human-readable format.
///
/// # Arguments
///
/// * `duration` - The Duration to format.
///
/// # Returns
///
/// A String representing the duration in seconds with microsecond precision.
fn format_duration(duration: Duration) -> String {
    format!("{:.6}s", duration.as_secs_f64())
}

/// Performs an extensive load test on the PebbleVault system.
///
/// This function creates multiple regions, adds objects with custom data, persists data, loads it,
/// deletes objects, and re-adds them to stress test the system thoroughly. It simulates
/// a high-load scenario that might be encountered in a real-world application, with a focus
/// on custom data integrity.
///
/// # Arguments
///
/// * `vault_manager` - A mutable reference to the VaultManager instance.
/// * `num_objects` - The number of objects to add in each test cycle.
/// * `num_regions` - The number of regions to create or use.
/// * `num_operations` - The number of additional operations to perform (delete/add cycles).
///
/// # Returns
///
/// * `Result<(), String>` - Ok if the load test completes successfully, or an error message if it fails.
///
/// # Examples
///
/// ```
/// let mut vault_manager = VaultManager::new("test_db.sqlite").unwrap();
/// run_load_test(&mut vault_manager, 10000, 5, 10).expect("Load test failed");
/// ```
pub fn run_load_test(vault_manager: &mut VaultManager<LoadTestData>, num_objects: usize, num_regions: usize, num_operations: usize) -> Result<(), String> {
    // Print the header for the load test
    println!("\n{}", "==== Running Enhanced PebbleVault Load Test ====".green().bold());
    println!("Number of objects to add: {}", num_objects.to_string().cyan());
    println!("Number of regions: {}", num_regions.to_string().cyan());
    println!("Number of additional operations: {}", num_operations.to_string().cyan());

    // Record the start time of the load test
    let start_time = Instant::now();

    // Get existing regions or create new ones if needed
    let regions: Vec<Uuid> = {
        // Get the list of existing regions
        let existing_regions: Vec<Uuid> = vault_manager.regions.keys().cloned().collect();
        if existing_regions.len() < num_regions {
            // If we don't have enough regions, create new ones
            let mut regions = existing_regions;
            for i in regions.len()..num_regions {
                let center = [i as f64 * 1000.0, 0.0, 0.0];
                let radius = 500.0;
                let region_id = vault_manager.create_or_load_region(center, radius)?;
                regions.push(region_id);
            }
            regions
        } else {
            // If we have enough regions, use the existing ones
            existing_regions
        }
    };

    println!("Using {} regions", regions.len());

    // Count existing objects across all regions
    let mut total_objects = 0;
    for &region_id in &regions {
        total_objects += vault_manager.query_region(region_id, -500.0, -500.0, -500.0, 500.0, 500.0, 500.0)?.len();
    }
    println!("Found {} existing objects", total_objects.to_string().cyan());

    // Define a closure to add objects to the VaultManager
    println!("\n{}", "Adding new objects with custom data".blue());
    let add_objects = |vm: &mut VaultManager<LoadTestData>, count: usize, regions: &[Uuid]| -> Result<Vec<Uuid>, String> {
        let mut rng = rand::thread_rng();
        let mut object_ids = Vec::with_capacity(count);
        let add_objects_start = Instant::now();

        // Create a progress bar for adding objects
        let pb = ProgressBar::new(count as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"));

        // Add the specified number of objects
        for _ in 0..count {
            let region_id = regions[rng.gen_range(0..regions.len())];
            let x = rng.gen_range(-500.0..500.0);
            let y = rng.gen_range(-500.0..500.0);
            let z = rng.gen_range(-500.0..500.0);
            let custom_data = Arc::new(LoadTestData::new_random());
            let object_uuid = Uuid::new_v4();
            let object_type = match rng.gen_range(0..3) {
                0 => "player",
                1 => "building",
                _ => "resource",
            };
            vm.add_object(region_id, object_uuid, object_type, x, y, z, custom_data)?;
            object_ids.push(object_uuid);
            pb.inc(1);
        }

        // Finish the progress bar
        pb.finish_with_message("Objects added");
        let add_objects_duration = add_objects_start.elapsed();
        println!("Added {} objects in {}", count, format_duration(add_objects_duration).green());
        println!("Average time per object: {}", format_duration(add_objects_duration / count as u32).yellow());
        Ok(object_ids)
    };

    // Add new objects to the VaultManager
    let mut new_object_ids = add_objects(vault_manager, num_objects, &regions)?;
    total_objects += new_object_ids.len();

    // Verify persistence and custom data integrity of added objects
    println!("\n{}", "Verifying persistence and custom data integrity".blue());
    let verify_start = Instant::now();
    for (i, &region_id) in regions.iter().enumerate() {
        match vault_manager.query_region(region_id, -500.0, -500.0, -500.0, 500.0, 500.0, 500.0) {
            Ok(objs) => {
                println!("Region {} (ID: {}) contains {} objects", i, region_id, objs.len().to_string().cyan());
                // Print details of up to 10 objects
                for (j, obj) in objs.iter().take(10).enumerate() {
                    println!("  Object {}: UUID: {}, Type: {}, Data: {:?}", j + 1, obj.uuid, obj.object_type, obj.custom_data);
                    // Verify custom data integrity
                    if obj.custom_data.level < 1 || obj.custom_data.level > 100 {
                        return Err(format!("Invalid level for object {}: {}", obj.uuid, obj.custom_data.level));
                    }
                    if obj.custom_data.health < 0.0 || obj.custom_data.health > 100.0 {
                        return Err(format!("Invalid health for object {}: {}", obj.uuid, obj.custom_data.health));
                    }
                }
                if objs.len() > 10 {
                    println!("  ... and {} more objects", objs.len() - 10);
                }
            },
            Err(e) => return Err(format!("Failed to query region {} (ID: {}): {}", i, region_id, e)),
        };
    }
    let verify_duration = verify_start.elapsed();
    println!("Persistence and custom data integrity verification completed in {}", format_duration(verify_duration).green());

    println!("Total objects after persistence: {}", total_objects.to_string().cyan());
    println!("Newly added objects: {}", num_objects.to_string().cyan());

    // Perform additional operations (delete and add cycles)
    println!("\n{}", "Performing additional operations".blue());
    let mut rng = rand::thread_rng();
    for op in 0..num_operations {
        println!("\n{}", format!("Performing operation set {}", op + 1).yellow());

        // Delete a random number of objects
        let num_to_delete = rng.gen_range(1..=new_object_ids.len() / 10);
        println!("Deleting {} objects", num_to_delete.to_string().cyan());
        let delete_start = Instant::now();
        let mut deleted_count = 0;
        for _ in 0..num_to_delete {
            if let Some(id) = new_object_ids.pop() {
                if let Err(e) = vault_manager.remove_object(id) {
                    println!("{}", format!("Failed to delete object {}: {}", id, e).red());
                } else {
                    deleted_count += 1;
                    total_objects -= 1;
                }
            }
        }
        let delete_duration = delete_start.elapsed();
        println!("Successfully deleted {} objects in {}", deleted_count.to_string().cyan(), format_duration(delete_duration).green());

        // Add new objects
        let num_to_add = rng.gen_range(1..=num_objects / 5);
        println!("Adding {} new objects", num_to_add.to_string().cyan());
        let new_objects = add_objects(vault_manager, num_to_add, &regions)?;
        new_object_ids.extend(new_objects.clone());
        total_objects += new_objects.len();

        // Verify persistence after changes
        println!("Verifying persistence after changes");
        let verify_changes_start = Instant::now();
        let verified_total_objects = regions.iter().map(|&region_id| {
            vault_manager.query_region(region_id, -500.0, -500.0, -500.0, 500.0, 500.0, 500.0)
                .map(|objects| objects.len())
                .unwrap_or(0)
        }).sum::<usize>();
        let verify_changes_duration = verify_changes_start.elapsed();

        println!("Total objects after operation set {}: {}", op + 1, verified_total_objects.to_string().cyan());
        println!("Expected object count: {}", total_objects.to_string().cyan());
        println!("Verification completed in {}", format_duration(verify_changes_duration).green());
        
        // Check if the verified count matches the expected count
        if verified_total_objects != total_objects {
            return Err(format!("Number of retrieved objects ({}) doesn't match the expected count ({})", verified_total_objects, total_objects).red().to_string());
        }
    }

    // Test retrieval and modification of custom data
    println!("\n{}", "Testing retrieval and modification of custom data".blue());
    test_custom_data_operations(vault_manager, &new_object_ids)?;

    // Test retrieval of players within a radius
    println!("\n{}", "Testing retrieval of players within a radius".blue());
    test_retrieve_players_within_radius(vault_manager, &regions)?;

    // Calculate and print final statistics
    let duration = start_time.elapsed();
    println!("\n{}", "Enhanced load test completed".green().bold());
    println!("Total time: {}", format_duration(duration).green());
    println!("Final object count: {}", total_objects.to_string().cyan());
    println!("Objects per second: {:.2}", (num_objects as f64 / duration.as_secs_f64()).to_string().cyan());

    Ok(())
}

/// Test function to retrieve and modify custom data of objects
///
/// This function selects random objects, retrieves their custom data, modifies it,
/// and then verifies that the changes were successfully applied.
///
/// # Arguments
///
/// * `vault_manager` - A mutable reference to the VaultManager instance.
/// * `object_ids` - A slice of object UUIDs to choose from.
///
/// # Returns
///
/// * `Result<(), String>` - Ok if the operations are successful, or an error message if they fail.
fn test_custom_data_operations(vault_manager: &mut VaultManager<LoadTestData>, object_ids: &[Uuid]) -> Result<(), String> {
    let mut rng = rand::thread_rng();
    let num_tests = std::cmp::min(10, object_ids.len());
    
    println!("Performing custom data operations on {} random objects", num_tests);

    for i in 0..num_tests {
        let object_id = object_ids[rng.gen_range(0..object_ids.len())];
        
        // Retrieve the object
        let mut object = vault_manager.get_object(object_id)?
            .ok_or_else(|| format!("Object not found: {}", object_id))?;

        println!("Test {}: Operating on object {}", i + 1, object_id);
        println!("  Original data: {:?}", object.custom_data);

        // Modify the custom data
        let mut new_data = (*object.custom_data).clone();
        new_data.level += 1;
        new_data.health = 100.0;
        new_data.inventory.push("New_Item".to_string());
        new_data.is_active = !new_data.is_active;

        // Update the object with new custom data
        object.custom_data = Arc::new(new_data.clone());
        vault_manager.update_object(&object)?;

        // Retrieve the object again to verify changes
        let updated_object = vault_manager.get_object(object_id)?
            .ok_or_else(|| format!("Updated object not found: {}", object_id))?;

        println!("  Updated data: {:?}", updated_object.custom_data);

        // Verify that the changes were applied correctly
        if *updated_object.custom_data != new_data {
            return Err(format!("Custom data mismatch for object {}", object_id));
        }
    }

    println!("Custom data operations completed successfully");
    Ok(())
}

/// Test function to retrieve players within a specified radius
///
/// This function selects a random region, generates a random point within that region,
/// and then retrieves all players within a specified radius of that point. It simulates
/// a common game scenario where nearby players need to be identified.
///
/// # Arguments
///
/// * `vault_manager` - A reference to the VaultManager instance.
/// * `regions` - A slice of region UUIDs to choose from.
///
/// # Returns
///
/// * `Result<(), String>` - Ok if the retrieval is successful, or an error message if it fails.
fn test_retrieve_players_within_radius(vault_manager: &VaultManager<LoadTestData>, regions: &[Uuid]) -> Result<(), String> {
    let mut rng = rand::thread_rng();
    // Select a random region
    let test_region = regions[rng.gen_range(0..regions.len())];
    // Generate a random point within the region
    let center_x = rng.gen_range(-500.0..500.0);
    let center_y = rng.gen_range(-500.0..500.0);
    let center_z = rng.gen_range(-500.0..500.0);
    let radius = 200.0;

    println!("Retrieving players within a radius of {} from point [{}, {}, {}] in region {}", 
             radius, center_x, center_y, center_z, test_region);

    let start_time = Instant::now();
    // Query the region for objects within the specified radius
    let objects = vault_manager.query_region(
        test_region, 
        center_x - radius, center_y - radius, center_z - radius,
        center_x + radius, center_y + radius, center_z + radius
    )?;

    // Filter the objects to get only players
    let players: Vec<&SpatialObject<LoadTestData>> = objects.iter()
        .filter(|obj| obj.object_type == "player")
        .collect();

    let duration = start_time.elapsed();
    println!("Retrieved {} total objects, {} of which are players, in {}", 
             objects.len(), players.len(), format_duration(duration).green());

    // Print details of up to 10 players
    for (i, player) in players.iter().take(10).enumerate() {
        println!("Player {}: UUID: {}, Position: {:?}, Data: {:?}", 
                 i + 1, player.uuid, player.point, player.custom_data);
    }
    if players.len() > 10 {
        println!("... and {} more players", players.len() - 10);
    }

    Ok(())
}

// Add this new arbitrary struct for testing
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
struct ArbitraryTestData {
    field1: String,
    field2: i32,
    field3: bool,
    field4: Vec<f64>,
}

// Implement random generation for ArbitraryTestData
impl Distribution<ArbitraryTestData> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ArbitraryTestData {
        ArbitraryTestData {
            field1: format!("Random_{}", rng.gen::<u32>()),
            field2: rng.gen(),
            field3: rng.gen(),
            field4: (0..rng.gen_range(1..5)).map(|_| rng.gen()).collect(),
        }
    }
}

/// Performs a load test using an arbitrary struct as custom data
pub fn run_arbitrary_data_load_test(num_objects: usize, num_regions: usize) -> Result<(), String> {
    println!("\n{}", "==== Running PebbleVault Load Test with Arbitrary Data ====".green().bold());
    
    let db_path = "arbitrary_test.db";
    let mut vault_manager: VaultManager<ArbitraryTestData> = VaultManager::new(db_path)
        .map_err(|e| format!("Failed to create VaultManager: {}", e))?;

    let start_time = Instant::now();

    // Create regions
    let regions: Vec<Uuid> = (0..num_regions)
        .map(|i| {
            let center = [i as f64 * 1000.0, 0.0, 0.0];
            let radius = 500.0;
            vault_manager.create_or_load_region(center, radius)
                .map_err(|e| format!("Failed to create region: {}", e))
        })
        .collect::<Result<Vec<Uuid>, String>>()?;

    println!("Created {} regions", regions.len());

    // Add objects with arbitrary data
    println!("\n{}", "Adding objects with arbitrary custom data".blue());
    let mut rng = rand::thread_rng();
    let pb = ProgressBar::new(num_objects as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));

    for _ in 0..num_objects {
        let region_id = regions[rng.gen_range(0..regions.len())];
        let x = rng.gen_range(-500.0..500.0);
        let y = rng.gen_range(-500.0..500.0);
        let z = rng.gen_range(-500.0..500.0);
        let custom_data = Arc::new(rng.gen::<ArbitraryTestData>());
        let object_uuid = Uuid::new_v4();
        let object_type = match rng.gen_range(0..3) {
            0 => "player",
            1 => "building",
            _ => "resource",
        };
        vault_manager.add_object(region_id, object_uuid, object_type, x, y, z, custom_data)
            .map_err(|e| format!("Failed to add object: {}", e))?;
        pb.inc(1);
    }
    pb.finish_with_message("Objects added");

    // Verify data
    println!("\n{}", "Verifying arbitrary custom data".blue());
    for (i, &region_id) in regions.iter().enumerate() {
        let objects = vault_manager.query_region(region_id, -500.0, -500.0, -500.0, 500.0, 500.0, 500.0)
            .map_err(|e| format!("Failed to query region {}: {}", i, e))?;
        println!("Region {} (ID: {}) contains {} objects", i, region_id, objects.len());
        
        // Print details of up to 5 objects
        for (j, obj) in objects.iter().take(5).enumerate() {
            println!("  Object {}: UUID: {}, Type: {}, Data: {:?}", j + 1, obj.uuid, obj.object_type, obj.custom_data);
        }
        if objects.len() > 5 {
            println!("  ... and {} more objects", objects.len() - 5);
        }
    }

    // Perform some updates
    println!("\n{}", "Performing updates on arbitrary data".blue());
    let objects_to_update = vault_manager.query_region(regions[0], -500.0, -500.0, -500.0, 500.0, 500.0, 500.0)
        .map_err(|e| format!("Failed to query region for updates: {}", e))?;
    
    for obj in objects_to_update.iter().take(10) {
        let mut updated_data = (*obj.custom_data).clone();
        updated_data.field2 += 1;
        updated_data.field3 = !updated_data.field3;
        updated_data.field4.push(rng.gen());
        
        let mut updated_obj = obj.clone();
        updated_obj.custom_data = Arc::new(updated_data);
        vault_manager.update_object(&updated_obj)
            .map_err(|e| format!("Failed to update object {}: {}", obj.uuid, e))?;
        
        println!("Updated object {}: {:?}", obj.uuid, updated_obj.custom_data);
    }

    let duration = start_time.elapsed();
    println!("\n{}", "Arbitrary data load test completed".green().bold());
    println!("Total time: {}", format_duration(duration).green());
    println!("Objects per second: {:.2}", (num_objects as f64 / duration.as_secs_f64()).to_string().cyan());

    Ok(())
}