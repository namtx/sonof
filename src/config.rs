use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to find home directory")?;
    let config = home.join(".sonof");
    fs::create_dir_all(&config)?;
    Ok(config)
}

fn token_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("token"))
}

pub fn save_token(token: &str) -> Result<()> {
    let path = token_file()?;
    fs::write(&path, token.trim())?;
    Ok(())
}

pub fn load_token() -> Result<String> {
    let path = token_file()?;
    let token = fs::read_to_string(&path)
        .context("No token found. Please run 'sonof login --token YOUR_TOKEN' first")?;
    Ok(token.trim().to_string())
}

