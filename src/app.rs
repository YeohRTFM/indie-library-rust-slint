use crate::config::AppSettings;
use crate::errors::ScanWarning;
use crate::library::{FilterCriteria, LibraryFilter, LibraryScanner};
use crate::models::{Game, GameMetadata};
use crate::services::{GameLauncher, GameService};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

slint::include_modules!();

pub struct AppState {
    pub library_root: PathBuf,
    pub all_games: Vec<Game>,
    pub filtered_indices: Vec<usize>,
    pub platforms: Vec<String>,
    pub categories: Vec<String>,
    pub criteria: FilterCriteria,
    pub settings: AppSettings,
    pub warnings: Vec<ScanWarning>,
    pub selected_original_idx: Option<usize>,
    pub edit_target_idx: Option<usize>,
    pub delete_target_idx: Option<usize>,
}

pub struct Application {
    pub window: AppWindow,
    pub state: Rc<RefCell<AppState>>,
}

impl Application {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let window = AppWindow::new()?;
        let settings = AppSettings::load();
        let library_root = settings.library_path.clone().unwrap_or_else(|| PathBuf::from("racine_bibliotheque"));
        let state = Rc::new(RefCell::new(AppState {
            library_root,
            all_games: Vec::new(),
            filtered_indices: Vec::new(),
            platforms: Vec::new(),
            categories: Vec::new(),
            criteria: FilterCriteria::default(),
            settings,
            warnings: Vec::new(),
            selected_original_idx: None,
            edit_target_idx: None,
            delete_target_idx: None,
        }));
        let app = Self { window, state };
        app.setup_callbacks();
        app.load_and_refresh();
        Ok(app)
    }
}