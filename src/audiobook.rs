use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::types::BookDetails;

/// Compile individual chapter files into a single m4b audiobook file
pub async fn compile_to_m4b(
    book: &BookDetails,
    chapter_files: &[PathBuf],
    output_dir: &Path,
    bitrate: &str,
) -> Result<PathBuf> {
    compile_to_m4b_internal(book, chapter_files, output_dir, bitrate, None).await
}

/// Compile individual chapter files into a single m4b audiobook file with optional cover art
pub async fn compile_to_m4b_with_artwork(
    book: &BookDetails,
    chapter_files: &[PathBuf],
    output_dir: &Path,
    bitrate: &str,
    cover_path: Option<&Path>,
) -> Result<PathBuf> {
    compile_to_m4b_internal(book, chapter_files, output_dir, bitrate, cover_path).await
}

async fn compile_to_m4b_internal(
    book: &BookDetails,
    chapter_files: &[PathBuf],
    output_dir: &Path,
    bitrate: &str,
    cover_path: Option<&Path>,
) -> Result<PathBuf> {
    if chapter_files.is_empty() {
        anyhow::bail!("No chapter files to compile");
    }

    println!("\nCompiling chapters into m4b audiobook...");

    // Check if ffmpeg is installed
    check_ffmpeg_installed()?;

    // Create output filename
    let output_filename = sanitize_filename(&format!("{}.m4b", book.title));
    let output_path = output_dir.join(&output_filename);

    // Create a temporary file list for ffmpeg concat
    let concat_file = output_dir.join("ffmpeg_concat.txt");
    create_concat_file(&concat_file, chapter_files)?;

    // Create metadata file with chapters
    let metadata_file = output_dir.join("ffmpeg_metadata.txt");
    create_metadata_file(&metadata_file, book, chapter_files).await?;

    println!("Merging {} chapters...", chapter_files.len());
    println!("Encoding to AAC at {} bitrate...", bitrate);
    if cover_path.is_some() {
        println!("Adding book cover artwork...");
    }
    println!("This may take a while depending on the book length...");

    // Run ffmpeg to concatenate and add metadata
    // We need to re-encode to AAC for m4b/m4a container compatibility
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&concat_file)
        .arg("-i")
        .arg(&metadata_file);

    // Add cover art if provided
    if let Some(cover) = cover_path {
        if cover.exists() {
            cmd.arg("-i").arg(cover);
        }
    }

    cmd.arg("-map_metadata")
        .arg("1")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg(bitrate);

    // If cover art was added, include it as attached picture
    if cover_path.is_some() && cover_path.unwrap().exists() {
        cmd.arg("-map")
            .arg("0:a")
            .arg("-map")
            .arg("2:v")
            .arg("-c:v")
            .arg("copy")
            .arg("-disposition:v:0")
            .arg("attached_pic");
    }

    cmd.arg("-movflags")
        .arg("+faststart") // Optimize for streaming
        .arg("-y"); // Overwrite output file if it exists

    cmd.arg(&output_path);

    let status = cmd.status().context("Failed to execute ffmpeg")?;

    // Clean up temporary files
    let _ = std::fs::remove_file(&concat_file);
    let _ = std::fs::remove_file(&metadata_file);

    if !status.success() {
        anyhow::bail!("ffmpeg failed with status: {}", status);
    }

    println!("✓ Compiled to: {}", output_path.display());

    Ok(output_path)
}

fn check_ffmpeg_installed() -> Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .context("Failed to check ffmpeg. Is ffmpeg installed?")?;

    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg is not working properly. Please install ffmpeg and try again.\n\
            On macOS: brew install ffmpeg\n\
            On Linux: sudo apt install ffmpeg\n\
            On Windows: Download from https://ffmpeg.org/"
        );
    }

    Ok(())
}

fn create_concat_file(concat_file: &Path, chapter_files: &[PathBuf]) -> Result<()> {
    let mut content = String::new();
    for file in chapter_files {
        // ffmpeg concat requires absolute paths or properly escaped relative paths
        let abs_path = file
            .canonicalize()
            .context("Failed to get absolute path for chapter file")?;
        content.push_str(&format!("file '{}'\n", abs_path.display()));
    }

    std::fs::write(concat_file, content).context("Failed to write concat file")?;
    Ok(())
}

async fn create_metadata_file(
    metadata_file: &Path,
    book: &BookDetails,
    chapter_files: &[PathBuf],
) -> Result<()> {
    let mut content = String::new();

    // Add global metadata
    content.push_str(";FFMETADATA1\n");
    content.push_str(&format!("title={}\n", escape_metadata(&book.title)));

    // Add author if available
    if let Some(author) = &book.author {
        if let Some(author_str) = author.as_str() {
            content.push_str(&format!("artist={}\n", escape_metadata(author_str)));
            content.push_str(&format!("album_artist={}\n", escape_metadata(author_str)));
        }
    }

    // Add description as comment
    content.push_str(&format!("comment={}\n", escape_metadata(&book.description)));

    // Add genre from categories
    if !book.categories.is_empty() {
        let genre = book.categories[0].name.clone();
        content.push_str(&format!("genre={}\n", escape_metadata(&genre)));
    }

    content.push_str("media_type=2\n"); // Audiobook

    // Get chapter durations by probing files
    let mut current_time_ms: u64 = 0;

    for (idx, _chapter_file) in chapter_files.iter().enumerate() {
        let chapter = &book.chapters[idx];
        let duration_ms = (chapter.duration * 1000.0) as u64;

        // Add chapter marker
        content.push_str("\n[CHAPTER]\n");
        content.push_str("TIMEBASE=1/1000\n");
        content.push_str(&format!("START={}\n", current_time_ms));
        content.push_str(&format!("END={}\n", current_time_ms + duration_ms));
        content.push_str(&format!("title={}\n", escape_metadata(&chapter.name)));

        current_time_ms += duration_ms;
    }

    std::fs::write(metadata_file, content).context("Failed to write metadata file")?;
    Ok(())
}

fn escape_metadata(s: &str) -> String {
    // Escape special characters for ffmpeg metadata
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('=', "\\=")
        .replace(';', "\\;")
        .replace('#', "\\#")
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}
