use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

/// Add audio files to an Apple Music playlist
/// Creates the playlist if it doesn't exist
/// If replace is true, deletes and recreates the playlist if it exists
pub fn add_to_playlist(
    playlist_name: &str,
    file_paths: &[impl AsRef<Path>],
    replace: bool,
) -> Result<()> {
    // Check if we're on macOS
    #[cfg(not(target_os = "macos"))]
    {
        return Err(anyhow!(
            "Apple Music integration is only available on macOS"
        ));
    }

    #[cfg(target_os = "macos")]
    {
        // Check if Music app is available
        check_music_app()?;

        // Handle playlist creation/replacement
        if replace {
            delete_playlist_if_exists(playlist_name)?;
        }
        create_or_get_playlist(playlist_name)?;

        // Add each file to the playlist
        for file_path in file_paths {
            let path = file_path.as_ref();
            if !path.exists() {
                eprintln!(
                    "⚠ Warning: File not found, skipping: {}",
                    path.display()
                );
                continue;
            }

            add_file_to_playlist(playlist_name, path)?;
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn check_music_app() -> Result<()> {
    let script = r#"
        tell application "System Events"
            set appExists to exists application process "Music"
        end tell
        return appExists
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| anyhow!("Failed to check Music app: {}", e))?;

    if !output.status.success() {
        return Err(anyhow!("Music app is not available on this system"));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn create_or_get_playlist(playlist_name: &str) -> Result<()> {
    // Escape quotes in playlist name
    let escaped_name = playlist_name.replace('"', "\\\"");

    let script = format!(
        r#"
        tell application "Music"
            set playlistExists to false
            repeat with p in playlists
                if name of p is "{}" then
                    set playlistExists to true
                    exit repeat
                end if
            end repeat
            
            if not playlistExists then
                make new playlist with properties {{name:"{}"}}
            end if
        end tell
        "#,
        escaped_name, escaped_name
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| anyhow!("Failed to create/get playlist: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create playlist: {}", stderr));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn add_file_to_playlist(playlist_name: &str, file_path: &Path) -> Result<()> {
    // Escape quotes in playlist name and file path
    let escaped_name = playlist_name.replace('"', "\\\"");
    let escaped_path = file_path
        .to_string_lossy()
        .replace('"', "\\\"")
        .to_string();

    let script = format!(
        r#"
        tell application "Music"
            set thePlaylist to playlist "{}"
            set theFile to POSIX file "{}"
            
            try
                set theTrack to add theFile to thePlaylist
            on error errMsg
                error "Failed to add track: " & errMsg
            end try
        end tell
        "#,
        escaped_name, escaped_path
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| anyhow!("Failed to add file to playlist: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to add file to playlist: {}", stderr));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn delete_playlist_if_exists(playlist_name: &str) -> Result<()> {
    // Escape quotes in playlist name
    let escaped_name = playlist_name.replace('"', "\\\"");

    let script = format!(
        r#"
        tell application "Music"
            repeat with p in playlists
                if name of p is "{}" then
                    try
                        delete p
                    end try
                    exit repeat
                end if
            end repeat
        end tell
        "#,
        escaped_name
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| anyhow!("Failed to delete playlist: {}", e))?;

    // We don't return an error if deletion fails, as the playlist might not exist
    // or might be a built-in playlist that can't be deleted
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Note: Could not delete playlist (may not exist): {}", stderr);
    }

    Ok(())
}

