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
/// * `db_path` - The path to the database file.
/// * `num_objects` - The initial number of objects to create and persist.
/// * `num_regions` - The number of regions to create.
/// * `num_operations` - The number of additional operations to perform (delete/add cycles).
///
/// # Returns
///
/// A Result indicating success or an error message.
pub fn run_load_test(db_path: &str, num_objects: usize, num_regions: usize, num_operations: usize) -> Result<(), String> {
    println!("\n{}", "==== Running Enhanced PebbleVault Load Test ====".green().bold());
    println!("Database path: {}", db_path.yellow());
    println!("Initial number of objects: {}", num_objects.to_string().cyan());
    println!("Number of regions: {}", num_regions.to_string().cyan());
    println!("Number of additional operations: {}", num_operations.to_string().cyan());

    let start_time = Instant::now();

    println!("\n{}", "Creating VaultManager instance".blue());
    let create_vm_start = Instant::now();
    let mut vault_manager = VaultManager::new(db_path)?;
    let create_vm_duration = create_vm_start.elapsed();
    println!("VaultManager created in {}", format_duration(create_vm_duration).green());

    println!("\n{}", "Creating regions".blue());
    let create_regions_start = Instant::now();

    // DEBUG: Progress bar for region creation
    let pb = ProgressBar::new(num_regions as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));

    let regions: Vec<Uuid> = (0..num_regions)
        .map(|i| {
            let center = [i as f64 * 1000.0, 0.0, 0.0];
            let radius = 500.0;
            let region_id = vault_manager.create_or_load_region(center, radius).unwrap();
            // DEBUG: Increment progress bar
            pb.inc(1);
            region_id
        })
        .collect();

    // DEBUG: Finish progress bar
    pb.finish_with_message("Regions created");
    let create_regions_duration = create_regions_start.elapsed();
    println!("Created {} regions in {}", num_regions, format_duration(create_regions_duration).green());

    let add_objects = |vm: &mut VaultManager, count: usize, regions: &[Uuid]| -> Result<Vec<Uuid>, String> {
        let mut rng = rand::thread_rng();
        let mut object_ids = Vec::with_capacity(count);
        let add_objects_start = Instant::now();

        // DEBUG: Progress bar for adding objects
        let pb = ProgressBar::new(count as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"));

        for _ in 0..count {
            let region_id = regions[rng.gen_range(0..regions.len())];
            let x = rng.gen_range(-500.0..500.0);
            let y = rng.gen_range(-500.0..500.0);
            let z = rng.gen_range(-500.0..500.0);
            let data = format!("Object {}", object_ids.len());
            let object_uuid = Uuid::new_v4();
            vm.add_object(region_id, object_uuid, x, y, z, &data)?;
            object_ids.push(object_uuid);
            // DEBUG: Increment progress bar
            pb.inc(1);
        }

        // DEBUG: Finish progress bar
        pb.finish_with_message("Objects added");
        let add_objects_duration = add_objects_start.elapsed();
        println!("Added {} objects in {}", count, format_duration(add_objects_duration).green());
        println!("Average time per object: {}", format_duration(add_objects_duration / count as u32).yellow());
        Ok(object_ids)
    };

    println!("\n{}", "Adding initial objects".blue());
    let mut object_ids = add_objects(&mut vault_manager, num_objects, &regions)?;

    println!("\n{}", "Persisting initial data to disk".blue());
    let persist_start = Instant::now();
    match vault_manager.persist_to_disk() {
        Ok(_) => {
            let persist_duration = persist_start.elapsed();
            println!("Data persisted successfully in {}", format_duration(persist_duration).green());
        },
        Err(e) => return Err(format!("Failed to persist data: {}", e)),
    }

    println!("\n{}", "Creating new VaultManager to test persistence".blue());
    let new_vm_start = Instant::now();
    let mut new_vault_manager = VaultManager::new(db_path)?;
    let new_vm_duration = new_vm_start.elapsed();
    println!("New VaultManager created in {}", format_duration(new_vm_duration).green());

    println!("\n{}", "Checking loaded regions in new VaultManager:".blue());
    for (i, &region_id) in regions.iter().enumerate() {
        match new_vault_manager.get_region(region_id) {
            Some(_) => println!("Region {} (ID: {}) {}", i, region_id, "loaded successfully".green()),
            None => println!("Region {} (ID: {}) {}", i, region_id, "not found in new VaultManager".red()),
        }
    }

    println!("\n{}", "Verifying persistence".blue());
    let verify_start = Instant::now();
    let mut total_objects = 0;
    for (i, &region_id) in regions.iter().enumerate() {
        match new_vault_manager.query_region(region_id, -500.0, -500.0, -500.0, 500.0, 500.0, 500.0) {
            Ok(objs) => {
                println!("Region {} (ID: {}) contains {} objects", i, region_id, objs.len().to_string().cyan());
                total_objects += objs.len();
            },
            Err(e) => return Err(format!("Failed to query region {} (ID: {}): {}", i, region_id, e)),
        };
    }
    let verify_duration = verify_start.elapsed();
    println!("Persistence verification completed in {}", format_duration(verify_duration).green());

    println!("Total objects retrieved after initial persistence: {}", total_objects.to_string().cyan());
    if total_objects != num_objects {
        return Err(format!("Persistence verification failed. Expected {} objects, but found {}.", num_objects, total_objects).red().to_string());
    }

    println!("\n{}", "Performing additional operations".blue());
    let mut rng = rand::thread_rng();
    for op in 0..num_operations {
        println!("\n{}", format!("Performing operation set {}", op + 1).yellow());

        let num_to_delete = rng.gen_range(1..=num_objects / 10);
        println!("Deleting {} objects", num_to_delete.to_string().cyan());
        let delete_start = Instant::now();
        let mut deleted_count = 0;
        for _ in 0..num_to_delete {
            if let Some(id) = object_ids.pop() {
                if let Err(e) = new_vault_manager.remove_object(id) {
                    println!("{}", format!("Failed to delete object {}: {}", id, e).red());
                } else {
                    deleted_count += 1;
                }
            }
        }
        let delete_duration = delete_start.elapsed();
        println!("Successfully deleted {} objects in {}", deleted_count.to_string().cyan(), format_duration(delete_duration).green());

        let num_to_add = rng.gen_range(1..=num_objects / 5);
        println!("Adding {} new objects", num_to_add.to_string().cyan());
        let new_objects = add_objects(&mut new_vault_manager, num_to_add, &regions)?;
        object_ids.extend(new_objects);

        println!("Persisting changes to disk");
        let persist_changes_start = Instant::now();
        new_vault_manager.persist_to_disk()?;
        let persist_changes_duration = persist_changes_start.elapsed();
        println!("Changes persisted in {}", format_duration(persist_changes_duration).green());

        println!("Verifying persistence after changes");
        let verify_changes_start = Instant::now();
        let mut verify_vault_manager = VaultManager::new(db_path)?;
        total_objects = regions.iter().map(|&region_id| {
            verify_vault_manager.query_region(region_id, -500.0, -500.0, -500.0, 500.0, 500.0, 500.0)
                .map(|objects| objects.len())
                .unwrap_or(0)
        }).sum::<usize>();
        let verify_changes_duration = verify_changes_start.elapsed();

        println!("Total objects after operation set {}: {}", op + 1, total_objects.to_string().cyan());
        println!("Expected object count: {}", object_ids.len().to_string().cyan());
        println!("Verification completed in {}", format_duration(verify_changes_duration).green());
        
        if total_objects != object_ids.len() {
            return Err(format!("Number of retrieved objects ({}) doesn't match the expected count ({})", total_objects, object_ids.len()).red().to_string());
        }
    }

    let duration = start_time.elapsed();
    println!("\n{}", "Enhanced load test completed".green().bold());
    println!("Total time: {}", format_duration(duration).green());
    println!("Final object count: {}", total_objects.to_string().cyan());
    println!("Objects per second: {:.2}", (total_objects as f64 / duration.as_secs_f64()).to_string().cyan());

    Ok(())
}