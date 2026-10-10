pub mod data_storage;
pub mod dps_calculator;
pub mod ping_tracker;

// Owns the capture threads and the tokio channel they feed.
// AionPartyMeter owns its capture lifecycle in desktop.rs.
