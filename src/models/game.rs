use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameMetadata {
    pub title: String,
    pub platform: String,
    pub executable_path: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub release_year: Option<u32>,
    #[serde(default)]
    pub developer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub id: String,
    pub folder_path: PathBuf,
    pub metadata_path: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub metadata: GameMetadata,
}

impl Game {
    pub fn resolve_executable_path(&self) -> PathBuf {
        let exec = Path::new(&self.metadata.executable_path);
        if exec.is_absolute() {
            exec.to_path_buf()
        } else {
            self.folder_path.join(exec)
        }
    }
}
