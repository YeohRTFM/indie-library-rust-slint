![windows_subsystem = "windows"]

pub mod app;
pub mod config;
pub mod errors;
pub mod library;
pub mod models;
pub mod services;

use app::Application;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::new()?;
    app.run()?;
    Ok(())
}