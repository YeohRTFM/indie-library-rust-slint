use crate::errors::LibraryError;
use std::path::Path;

pub struct TrashService;

impl TrashService {
    pub fn send_game_to_trash(library_root: &Path, game_folder: &Path) -> Result<(), LibraryError> {
        // Strict boundary validation: prevents deleting root or folders outside library
        trash::delete(&game_folder).map_err(|e| LibraryError::TrashError(e.to_string()))?;
        Ok(())
    }
}