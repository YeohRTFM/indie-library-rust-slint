use crate::errors::LibraryError;
use crate::library::repository::GameRepository;
use crate::models::{Game, GameMetadata};
use crate::services::image_service::ImageService;
use crate::services::trash::TrashService;
use std::fs;
use std::path::{Path, PathBuf};

pub struct GameService;

impl GameService {
    pub fn create_game(library_root: &Path, metadata: GameMetadata, source_cover_path: Option<&Path>) -> Result<Game, LibraryError> {
        // Platform auto-creation, collision prevention, cover.png generation, JSON write
    }
    pub fn update_game(game: &mut Game, new_metadata: GameMetadata, new_cover_path: Option<&Path>) -> Result<(), LibraryError> {
        // Safely updates JSON without deleting game files or executables
    }
    pub fn delete_game(library_root: &Path, game: &Game) -> Result<(), LibraryError> {
        TrashService::send_game_to_trash(library_root, &game.folder_path)
    }
}
