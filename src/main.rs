#![windows_subsystem = "windows"]

use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::thread;

slint::include_modules!();

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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub library_path: Option<PathBuf>,
    pub view_mode: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            library_path: None,
            view_mode: "grid".to_string(),
        }
    }
}

impl AppSettings {
    pub fn config_file() -> PathBuf {
        PathBuf::from("indie_library_config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_file();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&data) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(Self::config_file(), json);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub games: Vec<Game>,
    pub platforms: Vec<String>,
    pub categories: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn scan_library(library_root: &Path) -> ScanResult {
    let mut games = Vec::new();
    let mut platforms_set = BTreeSet::new();
    let mut categories_set = BTreeSet::new();
    let mut warnings = Vec::new();

    if !library_root.exists() {
        let _ = fs::create_dir_all(library_root);
        warnings.push(format!("Dossier créé automatiquement : {}", library_root.display()));
        return ScanResult {
            games,
            platforms: Vec::new(),
            categories: Vec::new(),
            warnings,
        };
    }

    let platform_entries = match fs::read_dir(library_root) {
        Ok(entries) => entries,
        Err(e) => {
            warnings.push(format!("Erreur d'accès à la racine : {}", e));
            return ScanResult { games, platforms: Vec::new(), categories: Vec::new(), warnings };
        }
    };

    for platform_entry in platform_entries.flatten() {
        let platform_path = platform_entry.path();
        if !platform_path.is_dir() {
            continue;
        }

        let platform_name = match platform_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        platforms_set.insert(platform_name.clone());

        let game_entries = match fs::read_dir(&platform_path) {
            Ok(entries) => entries,
            Err(e) => {
                warnings.push(format!("Plateforme '{}' inaccessible : {}", platform_name, e));
                continue;
            }
        };

        for game_entry in game_entries.flatten() {
            let game_dir = game_entry.path();
            if !game_dir.is_dir() {
                continue;
            }

            let dir_files = match fs::read_dir(&game_dir) {
                Ok(entries) => entries.flatten().collect::<Vec<_>>(),
                Err(e) => {
                    warnings.push(format!("Dossier '{}' inaccessible : {}", game_dir.display(), e));
                    continue;
                }
            };

            let json_file = dir_files.iter().find(|e| {
                e.path().extension().map_or(false, |ext| ext == "json")
            });

            let json_path = match json_file {
                Some(f) => f.path(),
                None => {
                    warnings.push(format!("Aucun JSON dans '{}'", game_dir.display()));
                    continue;
                }
            };

            let metadata_content = match fs::read_to_string(&json_path) {
                Ok(c) => c,
                Err(e) => {
                    warnings.push(format!("Erreur lecture JSON '{}' : {}", json_path.display(), e));
                    continue;
                }
            };

            let metadata: GameMetadata = match serde_json::from_str(&metadata_content) {
                Ok(m) => m,
                Err(e) => {
                    warnings.push(format!("JSON invalide '{}' : {}", json_path.display(), e));
                    continue;
                }
            };

            for cat in &metadata.categories {
                if !cat.trim().is_empty() {
                    categories_set.insert(cat.trim().to_string());
                }
            }

            let cover_path = dir_files.iter().find_map(|e| {
                let p = e.path();
                let is_cover = p.file_stem().map_or(false, |stem| stem == "cover")
                    && p.extension().map_or(false, |ext| ext == "png" || ext == "jpg" || ext == "jpeg");
                if is_cover { Some(p) } else { None }
            });

            let id = game_dir.file_name().unwrap_or_default().to_string_lossy().to_string();

            games.push(Game {
                id,
                folder_path: game_dir,
                metadata_path: json_path,
                cover_path,
                metadata,
            });
        }
    }

    ScanResult {
        games,
        platforms: platforms_set.into_iter().collect(),
        categories: categories_set.into_iter().collect(),
        warnings,
    }
}

pub fn filter_indices(
    games: &[Game],
    query: &str,
    platform: &str,
    category: &str,
    sort_by: &str,
) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    let mut indices: Vec<usize> = games
        .iter()
        .enumerate()
        .filter(|(_, g)| {
            if platform != "all" && !g.metadata.platform.eq_ignore_ascii_case(platform) {
                return false;
            }
            if category != "all" {
                let has_cat = g.metadata.categories.iter().any(|c| c.eq_ignore_ascii_case(category));
                if !has_cat {
                    return false;
                }
            }
            if !q.is_empty() {
                let match_title = g.metadata.title.to_lowercase().contains(&q);
                let match_dev = g.metadata.developer.as_ref().map_or(false, |d| d.to_lowercase().contains(&q));
                let match_desc = g.metadata.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&q));
                let match_cat = g.metadata.categories.iter().any(|c| c.to_lowercase().contains(&q));
                if !match_title && !match_dev && !match_desc && !match_cat {
                    return false;
                }
            }
            true
        })
        .map(|(i, _)| i)
        .collect();

    match sort_by {
        "title" => {
            indices.sort_by(|&a, &b| games[a].metadata.title.to_lowercase().cmp(&games[b].metadata.title.to_lowercase()));
        }
        "year" => {
            indices.sort_by(|&a, &b| {
                let ya = games[a].metadata.release_year.unwrap_or(0);
                let yb = games[b].metadata.release_year.unwrap_or(0);
                yb.cmp(&ya)
            });
        }
        "platform" => {
            indices.sort_by(|&a, &b| games[a].metadata.platform.to_lowercase().cmp(&games[b].metadata.platform.to_lowercase()));
        }
        _ => {}
    }

    indices
}

pub struct AppState {
    pub library_root: PathBuf,
    pub all_games: Vec<Game>,
    pub filtered_indices: Vec<usize>,
    pub platforms: Vec<String>,
    pub categories: Vec<String>,
    pub warnings: Vec<String>,
    pub settings: AppSettings,
    pub deleting_game_idx: Option<usize>,
}

fn sync_ui(window: &AppWindow, state: &AppState) {
    let mut game_datas = Vec::new();
    let mut cover_imgs = Vec::new();

    for &idx in &state.filtered_indices {
        if let Some(game) = state.all_games.get(idx) {
            let cover_img = game.cover_path.as_ref()
                .and_then(|p| Image::load_from_path(p).ok())
                .unwrap_or_default();

            let year_str = game.metadata.release_year.map_or(String::new(), |y| y.to_string());
            let dev_str = game.metadata.developer.clone().unwrap_or_default();
            let desc_str = game.metadata.description.clone().unwrap_or_default();
            let cat_str = game.metadata.categories.join(", ");
            let has_cover = game.cover_path.is_some();
            let cover_p = game.cover_path.as_ref().map_or(String::new(), |p| p.to_string_lossy().to_string());
            let exec_p = game.resolve_executable_path().to_string_lossy().to_string();

            game_datas.push(GameData {
                id: SharedString::from(&game.id),
                title: SharedString::from(&game.metadata.title),
                platform: SharedString::from(&game.metadata.platform),
                developer: SharedString::from(dev_str),
                release_year: SharedString::from(year_str),
                categories: SharedString::from(cat_str),
                has_cover,
                cover_path: SharedString::from(cover_p),
                executable_path: SharedString::from(exec_p),
                description: SharedString::from(desc_str),
                original_index: idx as i32,
            });

            cover_imgs.push(cover_img);
        }
    }

    window.set_games(ModelRc::new(VecModel::from(game_datas)));
    window.set_cover_images(ModelRc::new(VecModel::from(cover_imgs)));
    window.set_total_games(state.all_games.len() as i32);
    window.set_warning_count(state.warnings.len() as i32);

    let mut plat_strings = vec![SharedString::from("all")];
    for p in &state.platforms {
        plat_strings.push(SharedString::from(p));
    }
    window.set_platforms(ModelRc::new(VecModel::from(plat_strings)));

    let mut cat_strings = vec![SharedString::from("all")];
    for c in &state.categories {
        cat_strings.push(SharedString::from(c));
    }
    window.set_categories(ModelRc::new(VecModel::from(cat_strings)));

    let warn_strings: Vec<SharedString> = state.warnings.iter().map(|w| SharedString::from(w)).collect();
    window.set_warning_messages(ModelRc::new(VecModel::from(warn_strings)));
}

fn rescan_and_sync(window: &AppWindow, state: &Rc<RefCell<AppState>>) {
    let (library_root, query, platform, category, sort_by) = {
        let s = state.borrow();
        (
            s.library_root.clone(),
            window.get_search_query().to_string(),
            window.get_selected_platform().to_string(),
            window.get_selected_category().to_string(),
            window.get_sort_by().to_string(),
        )
    };

    let scan = scan_library(&library_root);
    let filtered = filter_indices(&scan.games, &query, &platform, &category, &sort_by);

    {
        let mut s = state.borrow_mut();
        s.all_games = scan.games;
        s.platforms = scan.platforms;
        s.categories = scan.categories;
        s.warnings = scan.warnings;
        s.filtered_indices = filtered;
    }

    let s = state.borrow();
    sync_ui(window, &s);
}

fn refilter_and_sync(window: &AppWindow, state: &Rc<RefCell<AppState>>) {
    let query = window.get_search_query().to_string();
    let platform = window.get_selected_platform().to_string();
    let category = window.get_selected_category().to_string();
    let sort_by = window.get_sort_by().to_string();

    {
        let mut s = state.borrow_mut();
        s.filtered_indices = filter_indices(&s.all_games, &query, &platform, &category, &sort_by);
    }

    let s = state.borrow();
    sync_ui(window, &s);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = AppWindow::new()?;
    let settings = AppSettings::load();
    let library_root = settings.library_path.clone().unwrap_or_else(|| PathBuf::from("racine_bibliotheque"));

    let state = Rc::new(RefCell::new(AppState {
        library_root: library_root.clone(),
        all_games: Vec::new(),
        filtered_indices: Vec::new(),
        platforms: Vec::new(),
        categories: Vec::new(),
        warnings: Vec::new(),
        settings,
        deleting_game_idx: None,
    }));

    window.set_view_mode(SharedString::from(&state.borrow().settings.view_mode));
    window.set_settings_library_path(SharedString::from(library_root.to_string_lossy().to_string()));

    rescan_and_sync(&window, &state);

    // Recherche
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_search_changed(move |_| {
            if let Some(win) = w.upgrade() {
                refilter_and_sync(&win, &st);
            }
        });
    }

    // Filtre Plateforme
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_platform_filtered(move |p| {
            if let Some(win) = w.upgrade() {
                win.set_selected_platform(p);
                refilter_and_sync(&win, &st);
            }
        });
    }

    // Filtre Catégorie
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_category_filtered(move |c| {
            if let Some(win) = w.upgrade() {
                win.set_selected_category(c);
                refilter_and_sync(&win, &st);
            }
        });
    }

    // Mode d'affichage (Grille / Liste)
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_view_mode_toggled(move |mode| {
            if let Some(win) = w.upgrade() {
                win.set_view_mode(mode.clone());
                let mut s = st.borrow_mut();
                s.settings.view_mode = mode.to_string();
                s.settings.save();
            }
        });
    }

    // Tri
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_sort_changed(move |sort| {
            if let Some(win) = w.upgrade() {
                win.set_sort_by(sort);
                refilter_and_sync(&win, &st);
            }
        });
    }

    // Actualiser
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_refresh_library(move || {
            if let Some(win) = w.upgrade() {
                rescan_and_sync(&win, &st);
            }
        });
    }

    // Lancer un jeu
    {
        let st = state.clone();
        window.on_play_game(move |original_idx| {
            let s = st.borrow();
            if let Some(game) = s.all_games.get(original_idx as usize) {
                let exec = game.resolve_executable_path();
                if !exec.exists() {
                    eprintln!("Exécutable introuvable : {}", exec.display());
                    return;
                }
                let folder = game.folder_path.clone();
                thread::spawn(move || {
                    let mut cmd = Command::new(&exec);
                    cmd.current_dir(&folder);
                    let _ = cmd.spawn();
                });
            }
        });
    }

    // Dialogue Ajout
    {
        let w = window.as_weak();
        window.on_open_add_game(move || {
            if let Some(win) = w.upgrade() {
                win.set_form_is_edit(false);
                win.set_form_error(SharedString::from(""));
                win.set_form_title(SharedString::from(""));
                win.set_form_platform(SharedString::from(""));
                win.set_form_developer(SharedString::from(""));
                win.set_form_year(SharedString::from(""));
                win.set_form_description(SharedString::from(""));
                win.set_form_categories(SharedString::from(""));
                win.set_form_executable(SharedString::from(""));
                win.set_form_cover(SharedString::from(""));
                win.set_show_game_form(true);
            }
        });
    }

    // Parcourir Exécutable
    {
        let w = window.as_weak();
        window.on_browse_executable_clicked(move || {
            if let Some(file) = rfd::FileDialog::new()
                .add_filter("Exécutables & Fichiers de jeu", &["exe", "bat", "html", "htm"])
                .pick_file() {
                if let Some(win) = w.upgrade() {
                    win.set_form_executable(SharedString::from(file.to_string_lossy().to_string()));
                }
            }
        });
    }

    // Parcourir Jaquette
    {
        let w = window.as_weak();
        window.on_browse_cover_clicked(move || {
            if let Some(file) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                .pick_file() {
                if let Some(win) = w.upgrade() {
                    win.set_form_cover(SharedString::from(file.to_string_lossy().to_string()));
                }
            }
        });
    }

    // Enregistrer Formulaire Jeu (Ajout / Modif)
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_save_game_form(move || {
            if let Some(win) = w.upgrade() {
                let title = win.get_form_title().trim().to_string();
                let platform = win.get_form_platform().trim().to_string();
                let executable = win.get_form_executable().trim().to_string();

                if title.is_empty() || platform.is_empty() || executable.is_empty() {
                    win.set_form_error(SharedString::from("Titre, plateforme et exécutable sont obligatoires."));
                    return;
                }

                let categories: Vec<String> = win.get_form_categories()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let year = win.get_form_year().trim().parse::<u32>().ok();
                let developer = {
                    let d = win.get_form_developer().trim().to_string();
                    if d.is_empty() { None } else { Some(d) }
                };
                let description = {
                    let d = win.get_form_description().trim().to_string();
                    if d.is_empty() { None } else { Some(d) }
                };

                let meta = GameMetadata {
                    title: title.clone(),
                    platform: platform.clone(),
                    executable_path: executable,
                    categories,
                    description,
                    release_year: year,
                    developer,
                };

                let root = st.borrow().library_root.clone();
                let plat_dir = root.join(&platform);
                let _ = fs::create_dir_all(&plat_dir);

                let slug = title.to_lowercase().replace(' ', "_").replace(|c: char| !c.is_alphanumeric() && c != '_', "");
                let game_folder = plat_dir.join(&slug);
                let _ = fs::create_dir_all(&game_folder);

                let json_path = game_folder.join(format!("{}.json", slug));
                if let Ok(json_content) = serde_json::to_string_pretty(&meta) {
                    let _ = fs::write(&json_path, json_content);
                }

                let cover_src = win.get_form_cover().trim().to_string();
                if !cover_src.is_empty() {
                    let src_path = PathBuf::from(&cover_src);
                    if src_path.exists() {
                        let _ = fs::copy(&src_path, game_folder.join("cover.png"));
                    }
                }

                win.set_show_game_form(false);
                rescan_and_sync(&win, &st);
            }
        });
    }

    // Demande de suppression
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_request_delete_game(move || {
            if let Some(win) = w.upgrade() {
                let current_game = win.get_selected_game();
                let idx = current_game.original_index as usize;
                st.borrow_mut().deleting_game_idx = Some(idx);
                win.set_delete_target_title(current_game.title);
                win.set_show_delete_confirm(true);
            }
        });
    }

    // Confirmation suppression -> Corbeille système
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_confirm_delete_game(move || {
            if let Some(win) = w.upgrade() {
                let idx_opt = st.borrow_mut().deleting_game_idx.take();
                if let Some(idx) = idx_opt {
                    let s = st.borrow();
                    if let Some(game) = s.all_games.get(idx) {
                        if game.folder_path.starts_with(&s.library_root) {
                            let _ = trash::delete(&game.folder_path);
                        }
                    }
                }
                win.set_show_delete_confirm(false);
                win.set_show_details(false);
                rescan_and_sync(&win, &st);
            }
        });
    }

    // Parcourir dossier bibliothèque (Paramètres)
    {
        let w = window.as_weak();
        window.on_browse_library_path(move || {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                if let Some(win) = w.upgrade() {
                    win.set_settings_library_path(SharedString::from(folder.to_string_lossy().to_string()));
                }
            }
        });
    }

    // Sauvegarder Paramètres
    {
        let w = window.as_weak();
        let st = state.clone();
        window.on_save_settings(move || {
            if let Some(win) = w.upgrade() {
                let p = win.get_settings_library_path().trim().to_string();
                if !p.is_empty() {
                    let mut s = st.borrow_mut();
                    let new_path = PathBuf::from(p);
                    s.library_root = new_path.clone();
                    s.settings.library_path = Some(new_path);
                    s.settings.save();
                }
                win.set_show_settings(false);
                rescan_and_sync(&win, &st);
            }
        });
    }

    window.run()?;
    Ok(())
}
