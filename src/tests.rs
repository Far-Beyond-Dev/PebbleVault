//! Test suite for PebbleVault functionality.
//!
//! This module contains a series of tests to verify the correct operation
//! of the PebbleVault spatial database system.

use super::*;
use tempfile::tempdir;
use uuid::Uuid;

/// Runs the complete test suite for PebbleVault.
///
/// This function executes a series of tests to verify different aspects
/// of PebbleVault's functionality, including VaultManager creation,
/// region and object operations, querying, player transfer, and persistence.
///
/// # Returns
///
/// A Result indicating success or an error message if any test fails.
pub fn run_tests() -> Result<(), String> {
    // Print a header for the test suite
    println!("\n==== Running PebbleVault Test Suite ====\n");

    // Test VaultManager creation
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let db_path = temp_dir.path().join("test_db_creation.sqlite");
    test_vault_manager_creation(db_path.to_str().unwrap())?;

    // Test region creation and object addition
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let db_path = temp_dir.path().join("test_db_region.sqlite");
    test_region_and_object_operations(db_path.to_str().unwrap())?;

    // Test querying and player transfer
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let db_path = temp_dir.path().join("test_db_query.sqlite");
    test_querying_and_player_transfer(db_path.to_str().unwrap())?;

    // Test persistence
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let db_path = temp_dir.path().join("test_db_persistence.sqlite");
    test_persistence(db_path.to_str().unwrap())?;

    // Print a footer indicating all tests passed
    println!("\n==== All PebbleVault tests passed successfully! ====\n");
    Ok(())
}

/// Tests the creation of a VaultManager instance.
///
/// This function verifies that a VaultManager can be created successfully
/// and that it starts with an empty set of regions.
///
/// # Arguments
///
/// * `db_path` - The path to the test database file.
///
/// # Returns
///
/// A Result indicating success or an error message if the test fails.
fn test_vault_manager_creation(db_path: &str) -> Result<(), String> {
    println!("\n---- Testing VaultManager Creation ----");
    println!("Creating VaultManager with database path: {}", db_path);

    // Create a new VaultManager instance
    let vault_manager = VaultManager::new(db_path)?;
    println!("VaultManager created successfully");

    // Verify that the VaultManager starts with no regions
    println!("Checking if VaultManager's regions are empty");
    assert!(vault_manager.regions.is_empty(), "VaultManager should be created with empty regions");
    println!("VaultManager's regions are empty as expected");

    println!("VaultManager creation test passed");
    Ok(())
}

/// Tests region creation and object addition operations.
///
/// This function verifies that regions can be created, objects can be added to regions,
/// and that these objects can be retrieved through queries.
///
/// # Arguments
///
/// * `db_path` - The path to the test database file.
///
/// # Returns
///
/// A Result indicating success or an error message if the test fails.
fn test_region_and_object_operations(db_path: &str) -> Result<(), String> {
    println!("\n---- Testing Region Creation and Object Addition ----");
    println!("Creating VaultManager with database path: {}", db_path);

    // Create a new VaultManager instance
    let mut vault_manager = VaultManager::new(db_path)?;

    // Verify that the VaultManager starts with no regions
    println!("Verifying that VaultManager starts with no regions");
    assert!(vault_manager.regions.is_empty(), "VaultManager should start with no regions");
    println!("VaultManager starts with no regions as expected");

    // Create a new region
    println!("Creating a new region");
    let region_center = [0.0, 0.0, 0.0];
    let region_radius = 100.0;
    let region_id = vault_manager.create_or_load_region(region_center, region_radius)?;
    println!("Region created with ID: {}", region_id);

    // Verify that a region was created
    println!("Verifying that a region was created");
    assert_eq!(vault_manager.regions.len(), 1, "VaultManager should have one region after creation");
    println!("VaultManager has one region as expected");

    // Add objects to the region
    println!("Adding objects to the region");
    let object1_uuid = Uuid::new_v4();
    vault_manager.add_object(region_id, object1_uuid, "player", 10.0, 20.0, 30.0, "Object 1 data")?;
    println!("Added object 1 with UUID: {}", object1_uuid);

    let object2_uuid = Uuid::new_v4();
    vault_manager.add_object(region_id, object2_uuid, "resource", -10.0, -20.0, -30.0, "Object 2 data")?;
    println!("Added object 2 with UUID: {}", object2_uuid);

    // Query the region to verify objects were added
    println!("Querying the region to verify objects were added");
    let query_result = vault_manager.query_region(region_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    println!("Query returned {} objects", query_result.len());
    assert_eq!(query_result.len(), 2, "Query should return 2 objects");
    println!("Query returned the expected number of objects");

    println!("Region creation and object addition test passed");
    Ok(())
}

/// Tests querying and player transfer operations.
///
/// This function verifies that objects can be queried within regions and that
/// players can be transferred between regions.
///
/// # Arguments
///
/// * `db_path` - The path to the test database file.
///
/// # Returns
///
/// A Result indicating success or an error message if the test fails.
fn test_querying_and_player_transfer(db_path: &str) -> Result<(), String> {
    println!("\n---- Testing Querying and Player Transfer ----");
    
    // Clear any existing database
    println!("Clearing the persistent database before the test");
    std::fs::remove_file(db_path).ok();
    
    println!("Creating VaultManager with database path: {}", db_path);
    let mut vault_manager = VaultManager::new(db_path)?;

    // Create two regions
    println!("Creating two regions");
    let region1_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;
    println!("Created region 1 with ID: {}", region1_id);
    let region2_id = vault_manager.create_or_load_region([200.0, 200.0, 200.0], 100.0)?;
    println!("Created region 2 with ID: {}", region2_id);

    // Add a player to region 1
    println!("Adding a player to region 1");
    let player_uuid = Uuid::new_v4();
    vault_manager.add_object(region1_id, player_uuid, "player", 10.0, 10.0, 10.0, "Player data")?;
    println!("Added player with UUID: {}", player_uuid);

    // Query region 1 to verify player was added
    println!("Querying region 1 to verify player was added");
    let query_result = vault_manager.query_region(region1_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    println!("Number of objects in region 1: {}", query_result.len());
    for obj in &query_result {
        println!("Object: UUID: {}, Type: {}, Data: {}, Position: {:?}", obj.uuid, obj.object_type, obj.data, obj.point);
    }
    assert_eq!(query_result.len(), 1, "Query should return 1 object (player)");
    println!("Query returned the expected number of objects");

    // Transfer player to region 2
    println!("Transferring player to region 2");
    vault_manager.transfer_player(player_uuid, region1_id, region2_id)?;
    println!("Player transferred");

    // Query both regions to verify transfer
    println!("Querying both regions to verify transfer");
    let query_result1 = vault_manager.query_region(region1_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    println!("Number of objects in region 1 after transfer: {}", query_result1.len());
    assert_eq!(query_result1.len(), 0, "Region 1 should be empty after transfer");
    println!("Region 1 is empty as expected");

    let query_result2 = vault_manager.query_region(region2_id, 150.0, 150.0, 150.0, 250.0, 250.0, 250.0)?;
    println!("Number of objects in region 2 after transfer: {}", query_result2.len());
    for obj in &query_result2 {
        println!("Object in region 2: UUID: {}, Type: {}, Data: {}, Position: {:?}", obj.uuid, obj.object_type, obj.data, obj.point);
    }
    assert_eq!(query_result2.len(), 1, "Region 2 should contain the transferred player");
    println!("Region 2 contains the transferred player as expected");

    // Verify player's position after transfer
    println!("Verifying that the player's position has been updated");
    let transferred_player = &query_result2[0];
    assert_eq!(transferred_player.point, [200.0, 200.0, 200.0], "Player should be at the center of region 2");
    println!("Player's position has been updated correctly");

    println!("Querying and player transfer test passed");
    Ok(())
}


/// Tests data persistence operations.
///
/// This function verifies that data can be persisted to disk and retrieved
/// correctly when a new VaultManager instance is created.
///
/// # Arguments
///
/// * `db_path` - The path to the test database file.
///
/// # Returns
///
/// A Result indicating success or an error message if the test fails.
fn test_persistence(db_path: &str) -> Result<(), String> {
    println!("\n---- Testing Persistence ----");
    
    // Clear any existing database
    println!("Clearing the persistent database before the test");
    std::fs::remove_file(db_path).ok();
    
    {
        println!("Creating first VaultManager instance");
        let mut vault_manager = VaultManager::new(db_path)?;
        
        println!("Creating a region");
        let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;
        println!("Created region with ID: {}", region_id);
        
        println!("Adding an object to the region");
        let object_uuid = Uuid::new_v4();
        vault_manager.add_object(region_id, object_uuid, "building", 10.0, 20.0, 30.0, "Persistent object data")?;
        println!("Added object with UUID: {}", object_uuid);
        
        println!("Persisting data to disk");
        vault_manager.persist_to_disk()?;
        println!("Data persisted successfully");
    }

    println!("Creating a new VaultManager instance to test if data was persisted");
    let vault_manager = VaultManager::new(db_path)?;
    
    println!("Loading objects from the persistent database");
    let objects = vault_manager.persistent_db.get_points_within_radius(0.0, 0.0, 0.0, 100.0)
        .map_err(|e| format!("Failed to load objects from persistent database: {}", e))?;

    println!("Number of persisted objects: {}", objects.len());
    for (i, obj) in objects.iter().take(10).enumerate() {
        println!("Persisted object {}: UUID: {}, Type: {}, Data: {}, Position: [{}, {}, {}]", 
                 i + 1, obj.id.unwrap(), obj.data["type"], obj.data["data"], obj.x, obj.y, obj.z);
    }
    if objects.len() > 10 {
        println!("... and {} more objects", objects.len() - 10);
    }
    assert_eq!(objects.len(), 1, "Persisted object should be loaded");
    println!("Correct number of objects loaded from persistent storage");

    println!("Persistence test passed");
    Ok(())
}