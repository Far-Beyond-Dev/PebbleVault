//! Load testing module for PebbleVault.
//!
//! This module provides functionality to perform comprehensive load tests on the PebbleVault system,
//! including creating, adding, persisting, loading, deleting, and re-adding objects across multiple regions.

use super::*;
use std::time::{Instant, Duration};
use rand::Rng;
use uuid::Uuid;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

/// Formats a Duration into a string with seconds and microseconds.
fn format_duration(duration: Duration) -> String {
    format!("{:.6}s", duration.as_secs_f64())
}

/// Performs an extensive load test on the PebbleVault system.
///
/// This function creates multiple regions, adds objects, persists data, loads it,
/// deletes objects, and re-adds them to stress test the system thoroughly.
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
/// A Result indicating success or an error message.
pub fn run_load_test(vault_manager: &mut VaultManager, num_objects: usize, num_regions: usize, num_operations: usize) -> Result<(), String> {
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
    println!("\n{}", "Adding new objects".blue());
    let add_objects = |vm: &mut VaultManager, count: usize, regions: &[Uuid]| -> Result<Vec<Uuid>, String> {
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
            // Randomly select a region for the object
            let region_id = regions[rng.gen_range(0..regions.len())];
            // Generate random coordinates within the region
            let x = rng.gen_range(-500.0..500.0);
            let y = rng.gen_range(-500.0..500.0);
            let z = rng.gen_range(-500.0..500.0);
            // Create object data
            let data = format!("Object {}", object_ids.len());
            let object_uuid = Uuid::new_v4();
            // Randomly assign an object type
            let object_type = match rng.gen_range(0..3) {
                0 => "player",
                1 => "building",
                _ => "resource",
            };
            // Add the object to the VaultManager
            vm.add_object(region_id, object_uuid, object_type, x, y, z, &data)?;
            object_ids.push(object_uuid);
            // Persist each new object individually
            vm.persistent_db.add_point(&MySQLGeo::Point::new(
                Some(object_uuid),
                x,
                y,
                z,
                serde_json::json!({
                    "type": object_type,
                    "data": data,
                }),
            ), region_id).map_err(|e| format!("Failed to persist new object: {}", e))?;
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

    // Verify persistence of added objects
    println!("\n{}", "Verifying persistence".blue());
    let verify_start = Instant::now();
    for (i, &region_id) in regions.iter().enumerate() {
        match vault_manager.query_region(region_id, -500.0, -500.0, -500.0, 500.0, 500.0, 500.0) {
            Ok(objs) => {
                println!("Region {} (ID: {}) contains {} objects", i, region_id, objs.len().to_string().cyan());
                // Print details of up to 10 objects
                for (j, obj) in objs.iter().take(10).enumerate() {
                    println!("  Object {}: UUID: {}, Type: {}, Data: {}", j + 1, obj.uuid, obj.object_type, obj.data);
                }
                if objs.len() > 10 {
                    println!("  ... and {} more objects", objs.len() - 10);
                }
            },
            Err(e) => return Err(format!("Failed to query region {} (ID: {}): {}", i, region_id, e)),
        };
    }
    let verify_duration = verify_start.elapsed();
    println!("Persistence verification completed in {}", format_duration(verify_duration).green());

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
                    // Persist the deletion
                    vault_manager.persistent_db.remove_point(id)
                        .map_err(|e| format!("Failed to persist object deletion: {}", e))?;
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

/// Test function to retrieve players within a specified radius
///
/// This function selects a random region, generates a random point within that region,
/// and then retrieves all players within a specified radius of that point.
///
/// # Arguments
///
/// * `vault_manager` - A reference to the VaultManager instance.
/// * `regions` - A slice of region UUIDs to choose from.
///
/// # Returns
///
/// A Result indicating success or an error message.
fn test_retrieve_players_within_radius(vault_manager: &VaultManager, regions: &[Uuid]) -> Result<(), String> {
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
    let players: Vec<&SpatialObject> = objects.iter()
        .filter(|obj| obj.object_type == "player")
        .collect();

    let duration = start_time.elapsed();
    println!("Retrieved {} total objects, {} of which are players, in {}", 
             objects.len(), players.len(), format_duration(duration).green());

    // Print details of up to 10 players
    for (i, player) in players.iter().take(10).enumerate() {
        println!("Player {}: UUID: {}, Position: {:?}, Data: {}", 
                 i + 1, player.uuid, player.point, player.data);
    }
    if players.len() > 10 {
        println!("... and {} more players", players.len() - 10);
    }

    Ok(())
}