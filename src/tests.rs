use super::*;
use std::fs;
use tempfile::tempdir;
use uuid::Uuid;

/// Test suite for PebbleVault functionality
pub fn run_tests() -> Result<(), String> {
    // Print a header for the test suite
    println!("\n==== Running PebbleVault Test Suite ====\n");

    // Test VaultManager creation
    // Create a temporary directory for the test database
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    // Generate a path for the test database file
    let db_path = temp_dir.path().join("test_db_creation.sqlite");
    // Run the VaultManager creation test
    test_vault_manager_creation(db_path.to_str().unwrap())?;

    // Test region creation and object addition
    // Create another temporary directory for this test
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    // Generate a path for the test database file
    let db_path = temp_dir.path().join("test_db_region.sqlite");
    // Run the region and object operations test
    test_region_and_object_operations(db_path.to_str().unwrap())?;

    // Test querying and player transfer
    // Create another temporary directory for this test
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    // Generate a path for the test database file
    let db_path = temp_dir.path().join("test_db_query.sqlite");
    // Run the querying and player transfer test
    test_querying_and_player_transfer(db_path.to_str().unwrap())?;

    // Test persistence
    // Create another temporary directory for this test
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    // Generate a path for the test database file
    let db_path = temp_dir.path().join("test_db_persistence.sqlite");
    // Run the persistence test
    test_persistence(db_path.to_str().unwrap())?;

    // Print a footer indicating all tests passed
    println!("\n==== All PebbleVault tests passed successfully! ====\n");
    Ok(())
}

/// Test VaultManager creation
fn test_vault_manager_creation(db_path: &str) -> Result<(), String> {
    // Print a header for this test
    println!("\n---- Testing VaultManager Creation ----");
    // Log the database path being used
    println!("Creating VaultManager with database path: {}", db_path);
    // Create a new VaultManager instance
    let vault_manager = VaultManager::new(db_path)?;
    // Log successful creation
    println!("VaultManager created successfully");
    
    // Log the start of the regions check
    println!("Checking if VaultManager's regions are empty");
    // Assert that the regions HashMap is empty
    assert!(vault_manager.regions.is_empty(), "VaultManager should be created with empty regions");
    // Log successful check
    println!("VaultManager's regions are empty as expected");
    
    // Log test completion
    println!("VaultManager creation test passed");
    Ok(())
}

/// Test region creation and object addition
fn test_region_and_object_operations(db_path: &str) -> Result<(), String> {
    // Print a header for this test
    println!("\n---- Testing Region Creation and Object Addition ----");
    // Log the database path being used
    println!("Creating VaultManager with database path: {}", db_path);
    // Create a new VaultManager instance
    let mut vault_manager = VaultManager::new(db_path)?;

    // Log the start of the initial regions check
    println!("Verifying that VaultManager starts with no regions");
    // Assert that the regions HashMap is initially empty
    assert!(vault_manager.regions.is_empty(), "VaultManager should start with no regions");
    // Log successful check
    println!("VaultManager starts with no regions as expected");

    // Log the start of region creation
    println!("Creating a new region");
    // Define the center coordinates and radius for the new region
    let region_center = [0.0, 0.0, 0.0];
    let region_radius = 100.0;
    // Create a new region and get its ID
    let region_id = vault_manager.create_or_load_region(region_center, region_radius)?;
    // Log the created region's ID
    println!("Region created with ID: {}", region_id);

    // Log the start of the region creation check
    println!("Verifying that a region was created");
    // Assert that there is now one region in the VaultManager
    assert_eq!(vault_manager.regions.len(), 1, "VaultManager should have one region after creation");
    // Log successful check
    println!("VaultManager has one region as expected");

    // Log the start of object addition
    println!("Adding objects to the region");
    // Generate a UUID for the first object
    let object1_uuid = Uuid::new_v4();
    // Add the first object to the region
    vault_manager.add_object(region_id, object1_uuid, 10.0, 20.0, 30.0, "Object 1 data")?;
    // Log the added object's UUID
    println!("Added object 1 with UUID: {}", object1_uuid);

    // Generate a UUID for the second object
    let object2_uuid = Uuid::new_v4();
    // Add the second object to the region
    vault_manager.add_object(region_id, object2_uuid, -10.0, -20.0, -30.0, "Object 2 data")?;
    // Log the added object's UUID
    println!("Added object 2 with UUID: {}", object2_uuid);

    // Log the start of the region query
    println!("Querying the region to verify objects were added");
    // Query the region for all objects within a large bounding box
    let query_result = vault_manager.query_region(region_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    // Log the number of objects returned by the query
    println!("Query returned {} objects", query_result.len());
    // Assert that the query returned 2 objects
    assert_eq!(query_result.len(), 2, "Query should return 2 objects");
    // Log successful check
    println!("Query returned the expected number of objects");

    // Log test completion
    println!("Region creation and object addition test passed");
    Ok(())
}

/// Test querying and player transfer
fn test_querying_and_player_transfer(db_path: &str) -> Result<(), String> {
    // Print a header for this test
    println!("\n---- Testing Querying and Player Transfer ----");
    
    // Log the start of database clearing
    println!("Clearing the persistent database before the test");
    // Remove the existing database file, ignoring any errors if it doesn't exist
    std::fs::remove_file(db_path).ok();
    
    // Log the database path being used
    println!("Creating VaultManager with database path: {}", db_path);
    // Create a new VaultManager instance
    let mut vault_manager = VaultManager::new(db_path)?;

    // Log the start of region creation
    println!("Creating two regions");
    // Create the first region and get its ID
    let region1_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;
    // Log the created region's ID
    println!("Created region 1 with ID: {}", region1_id);
    // Create the second region and get its ID
    let region2_id = vault_manager.create_or_load_region([200.0, 200.0, 200.0], 100.0)?;
    // Log the created region's ID
    println!("Created region 2 with ID: {}", region2_id);

    // Log the start of player addition
    println!("Adding a player to region 1");
    // Generate a UUID for the player
    let player_uuid = Uuid::new_v4();
    // Add the player to region 1
    vault_manager.add_object(region1_id, player_uuid, 10.0, 10.0, 10.0, "Player data")?;
    // Log the added player's UUID
    println!("Added player with UUID: {}", player_uuid);

    // Log the start of the region 1 query
    println!("Querying region 1 to verify player was added");
    // Query region 1 for all objects within a large bounding box
    let query_result = vault_manager.query_region(region1_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    // Log the number of objects in region 1
    println!("Number of objects in region 1: {}", query_result.len());
    // Log details of each object in region 1
    for obj in &query_result {
        println!("Object: UUID: {}, Data: {}, Position: {:?}", obj.uuid, obj.data, obj.point);
    }
    // Assert that there is 1 object (the player) in region 1
    assert_eq!(query_result.len(), 1, "Query should return 1 object (player)");
    // Log successful check
    println!("Query returned the expected number of objects");

    // Log the start of player transfer
    println!("Transferring player to region 2");
    // Transfer the player from region 1 to region 2
    vault_manager.transfer_player(player_uuid, region1_id, region2_id)?;
    // Log successful transfer
    println!("Player transferred");

    // Log the start of post-transfer queries
    println!("Querying both regions to verify transfer");
    // Query region 1 again
    let query_result1 = vault_manager.query_region(region1_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;
    // Log the number of objects in region 1 after transfer
    println!("Number of objects in region 1 after transfer: {}", query_result1.len());
    // Assert that region 1 is now empty
    assert_eq!(query_result1.len(), 0, "Region 1 should be empty after transfer");
    // Log successful check
    println!("Region 1 is empty as expected");

    // Query region 2
    let query_result2 = vault_manager.query_region(region2_id, 150.0, 150.0, 150.0, 250.0, 250.0, 250.0)?;
    // Log the number of objects in region 2 after transfer
    println!("Number of objects in region 2 after transfer: {}", query_result2.len());
    // Log details of each object in region 2
    for obj in &query_result2 {
        println!("Object in region 2: UUID: {}, Data: {}, Position: {:?}", obj.uuid, obj.data, obj.point);
    }
    // Assert that there is 1 object (the transferred player) in region 2
    assert_eq!(query_result2.len(), 1, "Region 2 should contain the transferred player");
    // Log successful check
    println!("Region 2 contains the transferred player as expected");

    // Log the start of player position verification
    println!("Verifying that the player's position has been updated");
    // Get the transferred player object
    let transferred_player = &query_result2[0];
    // Assert that the player's position is at the center of region 2
    assert_eq!(transferred_player.point, [200.0, 200.0, 200.0], "Player should be at the center of region 2");
    // Log successful check
    println!("Player's position has been updated correctly");

    // Log test completion
    println!("Querying and player transfer test passed");
    Ok(())
}

/// Test persistence
fn test_persistence(db_path: &str) -> Result<(), String> {
    // Print a header for this test
    println!("\n---- Testing Persistence ----");
    
    // Log the start of database clearing
    println!("Clearing the persistent database before the test");
    // Remove the existing database file, ignoring any errors if it doesn't exist
    std::fs::remove_file(db_path).ok();
    
    {
        // Log the creation of the first VaultManager instance
        println!("Creating first VaultManager instance");
        // Create a new VaultManager instance
        let mut vault_manager = VaultManager::new(db_path)?;
        
        // Log the start of region creation
        println!("Creating a region");
        // Create a new region and get its ID
        let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;
        // Log the created region's ID
        println!("Created region with ID: {}", region_id);
        
        // Log the start of object addition
        println!("Adding an object to the region");
        // Generate a UUID for the object
        let object_uuid = Uuid::new_v4();
        // Add the object to the region
        vault_manager.add_object(region_id, object_uuid, 10.0, 20.0, 30.0, "Persistent object data")?;
        // Log the added object's UUID
        println!("Added object with UUID: {}", object_uuid);
        
        // Log the start of data persistence
        println!("Persisting data to disk");
        // Persist the data to disk
        vault_manager.persist_to_disk()?;
        // Log successful persistence
        println!("Data persisted successfully");
    }

    // Log the creation of a new VaultManager instance
    println!("Creating a new VaultManager instance to test if data was persisted");
    // Create a new VaultManager instance
    let vault_manager = VaultManager::new(db_path)?;
    
    // Log the start of data loading
    println!("Loading objects from the persistent database");
    // Load objects from the persistent database within a large radius
    let objects = vault_manager.persistent_db.get_points_within_radius(0.0, 0.0, 0.0, 100.0)
        .map_err(|e| format!("Failed to load objects from persistent database: {}", e))?;

    // Log the number of persisted objects
    println!("Number of persisted objects: {}", objects.len());
    // Log details of each persisted object
    for obj in &objects {
        println!("Persisted object: UUID: {}, Data: {}, Position: [{}, {}, {}]", 
                 obj.id.unwrap(), obj.data, obj.x, obj.y, obj.z);
    }
    // Assert that 1 object was loaded from persistent storage
    assert_eq!(objects.len(), 1, "Persisted object should be loaded");
    // Log successful check
    println!("Correct number of objects loaded from persistent storage");

    // Log test completion
    println!("Persistence test passed");
    Ok(())
}