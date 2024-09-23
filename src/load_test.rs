use super::*;
use std::time::Instant;
use rand::Rng;
use uuid::Uuid;

/// Performs a load test on the PebbleVault system.
///
/// This function creates a large number of objects, adds them to regions,
/// persists them to disk, and then retrieves them to verify persistence.
///
/// # Arguments
///
/// * `db_path` - The path to the database file.
/// * `num_objects` - The number of objects to create and persist.
/// * `num_regions` - The number of regions to create.
///
/// # Returns
///
/// A Result indicating success or an error message.
pub fn run_load_test(db_path: &str, num_objects: usize, num_regions: usize) -> Result<(), String> {
    println!("\n==== Running PebbleVault Load Test ====\n");
    println!("Database path: {}", db_path);
    println!("Number of objects: {}", num_objects);
    println!("Number of regions: {}", num_regions);

    let start_time = Instant::now();

    // Create VaultManager
    let mut vault_manager = VaultManager::new(db_path)?;

    // Create regions
    let mut regions = Vec::new();
    for i in 0..num_regions {
        let center = [i as f64 * 1000.0, 0.0, 0.0];
        let radius = 500.0;
        let region_id = vault_manager.create_or_load_region(center, radius)?;
        regions.push(region_id);
    }

    println!("Created {} regions", num_regions);

    // Add objects
    let mut rng = rand::thread_rng();
    for i in 0..num_objects {
        let region_index = rng.gen_range(0..num_regions);
        let region_id = regions[region_index];
        let x = rng.gen_range(-500.0..500.0);
        let y = rng.gen_range(-500.0..500.0);
        let z = rng.gen_range(-500.0..500.0);
        let data = format!("Object {}", i);
        let object_uuid = Uuid::new_v4();

        vault_manager.add_object(region_id, object_uuid, x, y, z, &data)?;

        if (i + 1) % 1000 == 0 || i + 1 == num_objects {
            println!("Added {} objects", i + 1);
        }
    }

    // Persist data
    println!("Persisting data to disk...");
    vault_manager.persist_to_disk()?;

    // Create a new VaultManager to test persistence
    println!("Creating new VaultManager to test persistence...");
    let new_vault_manager = VaultManager::new(db_path)?;

    // Verify persistence
    let mut total_objects = 0;
    for region_id in &regions {
        let objects = new_vault_manager.query_region(*region_id, -500.0, -500.0, -500.0, 500.0, 500.0, 500.0)?;
        total_objects += objects.len();
    }

    println!("Total objects retrieved: {}", total_objects);
    assert_eq!(total_objects, num_objects, "Number of retrieved objects doesn't match the number of added objects");

    let duration = start_time.elapsed();
    println!("\nLoad test completed in {:?}", duration);
    println!("Objects per second: {:.2}", num_objects as f64 / duration.as_secs_f64());

    Ok(())
}