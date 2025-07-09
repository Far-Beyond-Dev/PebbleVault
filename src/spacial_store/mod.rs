//! spacial_store
//!
//! Core spatial storage module for PebbleVault providing persistence backends,
//! spatial data types, and the main VaultManager for region and object management.

pub mod backend;
pub mod sqlite_backend;
pub mod postgres_backend;
pub mod mysql_backend;
pub mod types;
pub mod manager;
