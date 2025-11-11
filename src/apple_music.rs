use anyhow::Context as _;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Add audio files to an Apple Music playlist
/// Creates the playlist if it doesn't exist
/// Note: Always deletes and recreates the playlist to ensure a clean state
///
/// Note: This function is kept for backward compatibility.
/// Use `add_to_playlist_with_artwork` for better Apple Music integration.
#[allow(dead_code)]
pub fn add_to_playlist(
    playlist_name: &str,
    file_paths: &[impl AsRef<Path>],
    _replace: bool,
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

        // Always delete and recreate the playlist for Apple Music
        // This ensures a clean state and avoids duplicate tracks
        delete_playlist_if_exists(playlist_name)?;
        create_or_get_playlist(playlist_name)?;

        // Add each file to the playlist
        for file_path in file_paths {
            let path = file_path.as_ref();
            if !path.exists() {
                eprintln!("⚠ Warning: File not found, skipping: {}", path.display());
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
    let escaped_path = file_path.to_string_lossy().replace('"', "\\\"").to_string();

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
        eprintln!(
            "Note: Could not delete playlist (may not exist): {}",
            stderr
        );
    }

    Ok(())
}

/// Set artwork for a track in Apple Music using AppleScript
/// Check if a track exists in Apple Music library by its file path
#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn track_exists_in_library(track_file: &Path) -> Result<bool> {
    let track_path = track_file
        .canonicalize()
        .context("Failed to resolve track path")?
        .to_string_lossy()
        .replace('"', "\\\"")
        .to_string();

    let script = format!(
        r#"
        tell application "Music"
            set foundTrack to false
            repeat with t in (every file track)
                try
                    if (POSIX path of (get location of t)) is "{}" then
                        set foundTrack to true
                        exit repeat
                    end if
                end try
            end repeat
            return foundTrack
        end tell
        "#,
        track_path
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .context("Failed to check if track exists in library")?;

    if !output.status.success() {
        return Ok(false);
    }

    let result = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_lowercase();
    Ok(result == "true")
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn set_track_artwork(track_file: &Path, artwork_file: &Path) -> Result<()> {
    if !track_file.exists() {
        return Err(anyhow!("Track file not found: {}", track_file.display()));
    }

    if !artwork_file.exists() {
        return Err(anyhow!(
            "Artwork file not found: {}",
            artwork_file.display()
        ));
    }

    // Convert paths to POSIX format for AppleScript
    let track_path = track_file
        .canonicalize()
        .context("Failed to resolve track path")?
        .to_string_lossy()
        .replace('"', "\\\"")
        .to_string();

    let artwork_path = artwork_file
        .canonicalize()
        .context("Failed to resolve artwork path")?
        .to_string_lossy()
        .replace('"', "\\\"")
        .to_string();

    let script = format!(
        r#"
        tell application "Music"
            set trackFile to POSIX file "{}"
            set artworkFile to POSIX file "{}"
            
            -- Find the track by location
            set foundTrack to missing value
            repeat with t in (every file track)
                if (POSIX path of (get location of t)) is "{}" then
                    set foundTrack to t
                    exit repeat
                end if
            end repeat
            
            if foundTrack is not missing value then
                try
                    -- Read the artwork data
                    set artworkData to (read artworkFile as «class PNGf»)
                    
                    -- Set the artwork
                    set data of artwork 1 of foundTrack to artworkData
                on error
                    try
                        -- Try reading as JPEG if PNG fails
                        set artworkData to (read artworkFile as «class JPEG»)
                        set data of artwork 1 of foundTrack to artworkData
                    on error errMsg
                        error "Failed to set artwork: " & errMsg
                    end try
                end try
            else
                error "Track not found in Music library"
            end if
        end tell
        "#,
        track_path, artwork_path, track_path
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .context("Failed to execute AppleScript for setting artwork")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to set artwork via AppleScript: {}", stderr));
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn set_track_artwork(_track_file: &Path, _artwork_file: &Path) -> Result<()> {
    Err(anyhow!(
        "Apple Music artwork setting is only available on macOS"
    ))
}

/// Add files to playlist and set their artwork using batch processing
/// This is more efficient than setting artwork immediately after adding each file
/// Note: Always deletes and recreates the playlist to ensure a clean state
#[allow(dead_code)]
pub fn add_to_playlist_with_artwork(
    playlist_name: &str,
    files_with_artwork: &[(impl AsRef<Path>, impl AsRef<Path>)],
    _replace: bool,
) -> Result<()> {
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

        // Always delete and recreate the playlist for Apple Music
        // This ensures a clean state and avoids duplicate tracks
        delete_playlist_if_exists(playlist_name)?;
        create_or_get_playlist(playlist_name)?;

        // Phase 1: Add all files to playlist first (fast, no waiting)
        eprintln!("📥 Adding files to Apple Music playlist...");
        let mut files_to_process: Vec<(PathBuf, PathBuf)> = Vec::new();

        for (file_path, artwork_path) in files_with_artwork {
            let file = file_path.as_ref();
            let artwork = artwork_path.as_ref();

            if !file.exists() {
                eprintln!("⚠ Warning: File not found, skipping: {}", file.display());
                continue;
            }

            // Add file to playlist
            if let Err(e) = add_file_to_playlist(playlist_name, file) {
                eprintln!("⚠ Warning: Failed to add file to playlist: {}", e);
                continue;
            }

            // Track for artwork setting
            if artwork.exists() {
                files_to_process.push((file.to_path_buf(), artwork.to_path_buf()));
            }
        }

        if files_to_process.is_empty() {
            return Ok(());
        }

        // Phase 2: Wait for Apple Music to index the files
        eprintln!(
            "⏳ Checking Apple Music for {} tracks...",
            files_to_process.len()
        );

        // Check which tracks are already indexed
        let max_wait_time = std::time::Duration::from_secs(30);
        let check_interval = std::time::Duration::from_millis(500);
        let start_time = std::time::Instant::now();
        let mut pending_files = files_to_process.clone();
        let mut ready_files: Vec<(PathBuf, PathBuf)> = Vec::new();

        while !pending_files.is_empty() && start_time.elapsed() < max_wait_time {
            let mut still_pending = Vec::new();

            for (file, artwork) in pending_files {
                match track_exists_in_library(&file) {
                    Ok(true) => {
                        ready_files.push((file, artwork));
                    }
                    Ok(false) => {
                        still_pending.push((file, artwork));
                    }
                    Err(_) => {
                        // If check fails, assume it's not ready yet
                        still_pending.push((file, artwork));
                    }
                }
            }

            pending_files = still_pending;

            if !pending_files.is_empty() {
                std::thread::sleep(check_interval);
            }
        }

        // Report status
        if pending_files.is_empty() {
            eprintln!("✅ All tracks found in Apple Music library");
        } else {
            eprintln!(
                "⚠ {}/{} tracks not found yet (will still attempt to set artwork)",
                pending_files.len(),
                files_to_process.len()
            );
            // Add pending files to ready list to attempt artwork setting anyway
            ready_files.extend(pending_files);
        }

        // Phase 3: Set artwork for all files
        eprintln!("🎨 Setting artwork for {} tracks...", ready_files.len());
        let mut failed_count = 0;

        for (file, artwork) in &ready_files {
            // Try to set artwork with minimal retries
            let max_retries = 2;
            let mut success = false;

            for attempt in 1..=max_retries {
                if attempt > 1 {
                    // Only wait on retries
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }

                match set_track_artwork(file, artwork) {
                    Ok(_) => {
                        success = true;
                        break;
                    }
                    Err(_e) => {
                        // Silent retry, only report on final failure
                    }
                }
            }

            if !success {
                failed_count += 1;
                eprintln!(
                    "⚠ Failed to set artwork for {}, can add manually from: {}",
                    file.file_name().unwrap_or_default().to_string_lossy(),
                    artwork.display()
                );
            }
        }

        if failed_count == 0 {
            eprintln!("✅ Successfully set artwork for all tracks!");
        } else if failed_count < files_to_process.len() {
            eprintln!(
                "✅ Set artwork for {}/{} tracks ({} failed)",
                files_to_process.len() - failed_count,
                files_to_process.len(),
                failed_count
            );
        }

        Ok(())
    }
}
