use crate::errors::LibraryError;
use crate::models::Game;
use std::process::Command;
use std::thread;

pub struct GameLauncher;

impl GameLauncher {
    pub fn launch(game: &Game, extra_args: &[String]) -> Result<(), LibraryError> {
        let executable_path = game.resolve_executable_path();
        if !executable_path.exists() {
            return Err(LibraryError::ExecutableNotFound(executable_path));
        }
        let working_dir = game.folder_path.clone();
        let args = extra_args.to_vec();
        // Asynchronous non-blocking execution with std::process::Command
        thread::spawn(move || {
            let mut cmd = Command::new(&executable_path);
            cmd.current_dir(&working_dir);
            for arg in &args { cmd.arg(arg); }
            let _ = cmd.spawn();
        });
        Ok(())
    }
}