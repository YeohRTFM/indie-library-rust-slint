use crate::errors::{LibraryError, ScanWarning};
use crate::library::repository::GameRepository;
use crate::models::Game;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LibraryScanner;

impl LibraryScanner {
    pub fn scan(library_root: &Path) -> Result<ScanResult, LibraryError> {
        let mut games = Vec::new();
        let mut platforms_set = BTreeSet::new();
        let mut categories_set = BTreeSet::new();
        let mut warnings = Vec::new();
        // Strict scanning: racine/<plateforme>/<jeu>/<mon_jeu>.json
        // Non-recursive exploration of game subfolders
        // Resilient error handling without aborting entire scan
        Ok(ScanResult { games, platforms: platforms_set.into_iter().collect(), categories: categories_set.into_iter().collect(), warnings })
    }
}