//! PebbleVault: A spatial database and object management system for game worlds.
//!
//! This crate provides functionality for managing spatial data in game environments,
//! including object storage, querying, and persistence.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Import the MySQLGeo module for database operations
mod MySQLGeo;
// Import the structs module for data structures
mod structs;
// Import the vault_manager module for managing spatial data
mod vault_manager;

// Re-export structs and VaultManager for easier access
pub use structs::*;
pub use vault_manager::VaultManager;

// Make the tests module public
pub mod tests;

// Import the load_test module for performance testing
pub mod load_test;
