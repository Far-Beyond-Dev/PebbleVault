//! # PebbleVault Test Suite
//!
//! This module contains comprehensive tests for the PebbleVault spatial database system.
//! It verifies the functionality of VaultManager creation, region and object operations,
//! querying, player transfer, persistence, and the ability to work with arbitrary custom data types.
//!
//! ## Key Features Tested
//!
//! - VaultManager initialization
//! - Region creation and management
//! - Object addition and retrieval
//! - Spatial querying
//! - Player transfer between regions
//! - Data persistence and recovery
//! - Support for arbitrary custom data structures
//!
//! ## Test Structure
//!
//! The test suite is composed of several individual test functions, each focusing on a specific
//! aspect of the PebbleVault system. These tests use temporary databases to ensure isolation
//! and clean state between test runs.

use super::*;
use tempfile::tempdir;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use colored::*;
use serde_json;

/// Custom data structure for basic tests
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
struct TestCustomData {
    name: String,
    value: i32,
}


/// Arbitrary struct for testing VaultManager's ability to handle custom types
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
struct ArbitraryGameObject {
    id: u32,
    position: [f32; 3],
    health: f32,
    inventory: Vec<String>,
}

/// Runs the complete test suite for PebbleVault.
pub fn run_tests() -> Result<(), String> {
    // Print the header for the test suite
    println!("\n{}", "==== Running PebbleVault Test Suite ====".green().bold());

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

    // Test with arbitrary struct
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let db_path = temp_dir.path().join("test_db_arbitrary.sqlite");
    test_with_arbitrary_struct(db_path.to_str().unwrap())?;

    // Print a footer indicating all tests passed
    println!("\n{}", "==== All PebbleVault tests passed successfully! ====".green().bold());
    Ok(())
}


/// Tests the creation of a VaultManager instance.
fn test_vault_manager_creation(db_path: &str) -> Result<(), String> {
    // Print the test header
    println!("\n{}", "---- Testing VaultManager Creation ----".blue());
    
    // Create a new VaultManager instance
    let vault_manager: VaultManager<TestCustomData> = VaultManager::new(db_path)?;
    println!("{}", "VaultManager created successfully".green());

    // Assert that the VaultManager starts with no regions
    assert!(vault_manager.regions.is_empty(), "VaultManager should be created with empty regions");
    println!("{}", "VaultManager's regions are empty as expected".green());

    // Print test passed message
    println!("{}", "VaultManager creation test passed".green());
    Ok(())
}

/// Tests region creation and object addition operations.
fn test_region_and_object_operations(db_path: &str) -> Result<(), String> {
    // Print the test header
    println!("\n{}", "---- Testing Region Creation and Object Addition ----".blue());

    // Create a new VaultManager instance
    let mut vault_manager: VaultManager<TestCustomData> = VaultManager::new(db_path)?;

    // Assert that the VaultManager starts with no regions
    assert!(vault_manager.regions.is_empty(), "VaultManager should start with no regions");
    println!("{}", "VaultManager starts with no regions as expected".green());

    // Create a new cubic region
    let region_center = [0.0, 0.0, 0.0];
    let region_size = 100.0;  // 100x100x100 cube
    let region_id = vault_manager.create_or_load_region(region_center, region_size)?;
    println!("Created cubic region with ID: {}", region_id.to_string().cyan());

    // Assert that the VaultManager now has one region
    assert_eq!(vault_manager.regions.len(), 1, "VaultManager should have one region after creation");
    println!("{}", "VaultManager has one region as expected".green());

    // Add the first object to the region
    let object1_uuid = Uuid::new_v4();
    let custom_data1 = Arc::new(TestCustomData {
        name: "Object 1".to_string(),
        value: 42,
    });
    vault_manager.add_object(
        region_id,
        object1_uuid,
        "player",
        10.0,
        20.0,
        30.0,
        1.0, // size_x
        1.0, // size_y
        1.0, // size_z
        custom_data1,
    )?;
    println!("Added object 1 with UUID: {}", object1_uuid.to_string().cyan());

    // Add the second object to the region
    let object2_uuid = Uuid::new_v4();
    let custom_data2 = Arc::new(TestCustomData {
        name: "Object 2".to_string(),
        value: 100,
    });
    vault_manager.add_object(
        region_id,
        object2_uuid,
        "resource",
        -10.0,
        -20.0,
        -30.0,
        2.0,
        2.0,
        2.0,
        custom_data2,
    )?;
    println!("Added object 2 with UUID: {}", object2_uuid.to_string().cyan());

    // Query the region to verify object addition
    let query_result = vault_manager.query_region(region_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    println!("Query returned {} objects", query_result.len().to_string().cyan());
    assert_eq!(query_result.len(), 2, "Query should return 2 objects");
    println!("{}", "Query returned the expected number of objects".green());

    // Print test passed message
    println!("{}", "Region creation and object addition test passed".green());
    Ok(())
}


/// Tests querying and player transfer operations.
fn test_querying_and_player_transfer(db_path: &str) -> Result<(), String> {
    // Print the test header
    println!("\n{}", "---- Testing Querying and Player Transfer ----".blue());
    
    // Remove any existing database file
    std::fs::remove_file(db_path).ok();
    
    // Create a new VaultManager instance
    let mut vault_manager: VaultManager<TestCustomData> = VaultManager::new(db_path)?;

    // Create two cubic regions
    let region1_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;  // 100x100x100 cube
    println!("Created region 1 with ID: {}", region1_id.to_string().cyan());
    let region2_id = vault_manager.create_or_load_region([200.0, 200.0, 200.0], 100.0)?;  // 100x100x100 cube
    println!("Created region 2 with ID: {}", region2_id.to_string().cyan());

    // Add a player to region 1
    let player_uuid = Uuid::new_v4();
    let player_data = Arc::new(TestCustomData {
        name: "Player 1".to_string(),
        value: 50,
    });
    vault_manager.add_object(
        region1_id,
        player_uuid,
        "player",
        10.0, 10.0, 10.0,
        1.5, 1.5, 1.5,
        player_data,
    )?;
    println!("Added player with UUID: {}", player_uuid.to_string().cyan());

    // Query region 1 to verify player addition
    let query_result = vault_manager.query_region(region1_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    println!("Number of objects in region 1: {}", query_result.len().to_string().cyan());
    for obj in &query_result {
        println!("Object: UUID: {}, Type: {}, Custom Data: {:?}, Position: {:?}", 
                 obj.uuid.to_string().cyan(), obj.object_type, obj.custom_data, obj.point);
    }
    assert_eq!(query_result.len(), 1, "Query should return 1 object (player)");
    println!("{}", "Query returned the expected number of objects".green());

    // Transfer the player from region 1 to region 2
    vault_manager.transfer_player(player_uuid, region1_id, region2_id)?;
    println!("{}", "Player transferred".green());

    // Query region 1 to verify player removal
    let query_result1 = vault_manager.query_region(region1_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    println!("Number of objects in region 1 after transfer: {}", query_result1.len().to_string().cyan());
    assert_eq!(query_result1.len(), 0, "Region 1 should be empty after transfer");
    println!("{}", "Region 1 is empty as expected".green());

    // Query region 2 to verify player addition
    let query_result2 = vault_manager.query_region(region2_id, 150.0, 150.0, 150.0, 250.0, 250.0, 250.0)?;
    println!("Number of objects in region 2 after transfer: {}", query_result2.len().to_string().cyan());
    for obj in &query_result2 {
        println!("Object in region 2: UUID: {}, Type: {}, Custom Data: {:?}, Position: {:?}", 
                 obj.uuid.to_string().cyan(), obj.object_type, obj.custom_data, obj.point);
    }
    assert_eq!(query_result2.len(), 1, "Region 2 should contain the transferred player");
    println!("{}", "Region 2 contains the transferred player as expected".green());

    // Verify player's new position
    let transferred_player = &query_result2[0];
    assert_eq!(transferred_player.point, [200.0, 200.0, 200.0], "Player should be at the center of region 2");
    println!("{}", "Player's position has been updated correctly".green());

    // Print test passed message
    println!("{}", "Querying and player transfer test passed".green());
    Ok(())
}


/// Tests data persistence operations.
fn test_persistence(db_path: &str) -> Result<(), String> {
    // Print the test header
    println!("\n{}", "---- Testing Persistence ----".blue());
    
    // Remove any existing database file
    std::fs::remove_file(db_path).ok();
    
    {
        // Create a new VaultManager instance
        let mut vault_manager: VaultManager<TestCustomData> = VaultManager::new(db_path)?;
        
        // Create a cubic region
        let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;  // 100x100x100 cube
        println!("Created cubic region with ID: {}", region_id.to_string().cyan());
        
        // Add an object to the region
        let object_uuid = Uuid::new_v4();
        let custom_data = Arc::new(TestCustomData {
            name: "Persistent Object".to_string(),
            value: 200,
        });
        vault_manager.add_object(
            region_id,
            object_uuid,
            "building",
            10.0, 20.0, 30.0,
            2.0, 2.0, 2.0,
            custom_data,
        )?;
        println!("Added object with UUID: {}", object_uuid.to_string().cyan());

        // Persist data to disk
        vault_manager.persist_to_disk()?;
        println!("{}", "Data persisted successfully".green());
    }

    // Create a new VaultManager instance to load persisted data
    let vault_manager: VaultManager<TestCustomData> = VaultManager::new(db_path)?;
    
    // Retrieve persisted objects
    let objects = vault_manager.persistent_db.get_points_within_radius(0.0, 0.0, 0.0, 100.0)
        .map_err(|e| format!("Failed to load objects from persistent database: {}", e))?;

    // Verify persisted objects
    println!("Number of persisted objects: {}", objects.len().to_string().cyan());
    for (i, obj) in objects.iter().enumerate() {
        println!("Persisted object {}: UUID: {}, Type: {}, Custom Data: {:?}, Position: [{}, {}, {}]", 
                 i + 1, obj.id.unwrap().to_string().cyan(), obj.object_type, obj.custom_data, obj.x, obj.y, obj.z);
    }
    assert_eq!(objects.len(), 1, "Persisted object should be loaded");
    println!("{}", "Correct number of objects loaded from persistent storage".green());

    // Print test passed message
    println!("{}", "Persistence test passed".green());
    Ok(())
}


/// Tests VaultManager with an arbitrary struct as custom data.
fn test_with_arbitrary_struct(db_path: &str) -> Result<(), String> {
    // Print the test header
    println!("\n{}", "---- Testing VaultManager with Arbitrary Struct ----".blue());

    // Remove any existing database file
    std::fs::remove_file(db_path).ok();

    // Create a new VaultManager instance with ArbitraryGameObject as custom data
    let mut vault_manager: VaultManager<ArbitraryGameObject> = VaultManager::new(db_path)?;

    // Create a cubic region
    let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;  // 100x100x100 cube
    println!("Created cubic region with ID: {}", region_id.to_string().cyan());

    // Create an ArbitraryGameObject instance
    let game_object = Arc::new(ArbitraryGameObject {
        id: 1,
        position: [10.0, 20.0, 30.0],
        health: 100.0,
        inventory: vec!["Sword".to_string(), "Health Potion".to_string()],
    });

    // Add the game object to the region
    let object_uuid = Uuid::new_v4();
    vault_manager.add_object(
        region_id,
        object_uuid,
        "game_object",
        10.0, 20.0, 30.0,
        2.5, 3.0, 1.8,
        game_object.clone(),
    )?;
    println!("Added game object with UUID: {}", object_uuid.to_string().cyan());

    let query_result = vault_manager.query_region(region_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    println!("Query returned {} objects", query_result.len().to_string().cyan());
    assert_eq!(query_result.len(), 1, "Query should return 1 object");
    
    let retrieved_object = &query_result[0];
    assert_eq!(retrieved_object.uuid, object_uuid, "Retrieved object UUID should match");
    assert_eq!(*retrieved_object.custom_data, *game_object, "Retrieved custom data should match original");
    println!("{}", "Retrieved object matches the original".green());

    vault_manager.persist_to_disk()?;
    println!("{}", "Data persisted successfully".green());

    let new_vault_manager: VaultManager<ArbitraryGameObject> = VaultManager::new(db_path)?;
    let loaded_objects = new_vault_manager.persistent_db.get_points_within_radius(0.0, 0.0, 0.0, 100.0)
        .map_err(|e| format!("Failed to load objects from persistent database: {}", e))?;

    assert_eq!(loaded_objects.len(), 1, "Persisted object should be loaded");
    let loaded_object = &loaded_objects[0];
    let loaded_custom_data: ArbitraryGameObject = serde_json::from_value(loaded_object.custom_data.clone())
        .map_err(|e| format!("Failed to deserialize loaded custom data: {}", e))?;
    assert_eq!(loaded_custom_data, *game_object, "Loaded custom data should match original");
    println!("{}", "Loaded object matches the original".green());

    println!("{}", "VaultManager with arbitrary struct test passed".green());
    Ok(())
}