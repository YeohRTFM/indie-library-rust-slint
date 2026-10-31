use indie_library::library::{GameRepository, LibraryFilter, LibraryScanner, FilterCriteria};
use indie_library::models::{Game, GameMetadata};
use indie_library::services::{GameService, TrashService};
use tempfile::tempdir;

#[test]
fn test_metadata_serialization_roundtrip() { /* ... */ }
#[test]
fn test_scan_and_dynamic_platform_and_categories_discovery() { /* ... */ }
#[test]
fn test_scanner_resilience_to_invalid_json() { /* ... */ }
#[test]
fn test_case_insensitive_search_and_filters() { /* ... */ }
#[test]
fn test_relative_and_absolute_executable_resolution() { /* ... */ }
#[test]
fn test_game_creation_and_collision_prevention() { /* ... */ }
#[test]
fn test_security_protection_outside_library() { /* ... */ }
