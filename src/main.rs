use bsky_sdk::BskyAgent;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

mod app;
mod clearsky_v1;
mod db;
#[allow(dead_code)]
mod harness;
mod model;
mod net_backend;
mod post;
mod ui;
mod visualization;

use crate::app::{App, EvilGemmaConfig, restore_store_from_db, run_app};
use crate::db::AppDb;
use crate::harness::context_window_logger::reset_debug_dir;

const DEFAULT_DB_PATH: &str = "shock_absorber.sqlite3";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenvy::dotenv();

    let handle = env::var("BSKY_HANDLE")?;
    let password = env::var("BSKY_APP_PASSWORD")?;

    let agent = Arc::new(BskyAgent::builder().build().await?);
    agent.login(&handle, &password).await?;
    let evil_gemma = Arc::new(EvilGemmaConfig::from_env()?);
    let db_path =
        env::var("SHOCK_ABSORBER_DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string());
    let db = AppDb::new(resolve_db_path(db_path))?;
    let mut app = App::new(handle);
    restore_store_from_db(&mut app, &db)?;
    app.set_status(format!("{} | db {}", app.status(), db.path().display()));
    let debug_base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _ = reset_debug_dir(&debug_base_dir);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, agent, evil_gemma, app, &db).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn resolve_db_path(path: String) -> PathBuf {
    let candidate = PathBuf::from(&path);
    if candidate.is_absolute() {
        candidate
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(candidate)
    }
}
