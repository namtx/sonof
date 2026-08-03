use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::types::BookDetails;

const MAX_PART_SIZE_BYTES: u64 = 500 * 1024 * 1024; // 500 MB

/// Compile individual chapter files into one or more m4b audiobook files with chapter metadata.
/// If the estimated encoded size exceeds 500 MB the output is split into parts on chapter boundaries.
/// Returns all output paths produced.
pub async fn compile_to_m4b_with_artwork(
    book: &BookDetails,
    chapter_files: &[PathBuf],
    output_dir: &Path,
    bitrate: &str,
    cover_path: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    if chapter_files.is_empty() {
        anyhow::bail!("No chapter files to compile");
    }

    check_ffmpeg_installed()?;

    let parts = split_into_parts(book, chapter_files, bitrate)?;

    let mut output_paths = Vec::new();
    let total_parts = parts.len();

    for (part_idx, part_files) in parts.iter().enumerate() {
        // Slice the book chapters to those in this part
        let start_chapter = chapter_files
            .iter()
            .position(|f| f == part_files[0])
            .unwrap_or(0);
        let end_chapter = start_chapter + part_files.len();
        let part_chapters = &book.chapters[start_chapter..end_chapter];

        let suffix = if total_parts > 1 {
            format!(" Part {}", part_idx + 1)
        } else {
            String::new()
        };
        let output_filename =
            sanitize_filename(&format!("{}{}.m4b", book.title, suffix));
        let output_path = output_dir.join(&output_filename);

        let label = if total_parts > 1 {
            format!("📦 Joining part {}/{}...", part_idx + 1, total_parts)
        } else {
            "📦 Joining chapters into m4b...".to_string()
        };
        let total_duration_s: f64 = part_chapters.iter().map(|c| c.duration).sum();

        build_m4b(
            part_chapters,
            part_files,
            &output_path,
            output_dir,
            bitrate,
            cover_path,
            part_idx,
            &label,
            total_duration_s,
        )
        .await?;

        output_paths.push(output_path);
    }

    Ok(output_paths)
}

/// Decide how to split `chapter_files` into parts so each part's estimated encoded size ≤ 500 MB.
/// Estimation: bitrate_bps * total_duration_seconds / 8 bytes.
fn split_into_parts<'a>(
    book: &BookDetails,
    chapter_files: &'a [PathBuf],
    bitrate: &str,
) -> Result<Vec<Vec<&'a PathBuf>>> {
    let bitrate_bps = parse_bitrate_bps(bitrate)?;

    let mut parts: Vec<Vec<&PathBuf>> = Vec::new();
    let mut current_part: Vec<&PathBuf> = Vec::new();
    let mut current_duration_s: f64 = 0.0;

    for (idx, file) in chapter_files.iter().enumerate() {
        let chapter_duration_s = book.chapters[idx].duration;
        let part_duration_with_this = current_duration_s + chapter_duration_s;
        let estimated_bytes = (bitrate_bps as f64 * part_duration_with_this / 8.0) as u64;

        if !current_part.is_empty() && estimated_bytes > MAX_PART_SIZE_BYTES {
            parts.push(current_part);
            current_part = Vec::new();
            current_duration_s = 0.0;
        }

        current_part.push(file);
        current_duration_s += chapter_duration_s;
    }

    if !current_part.is_empty() {
        parts.push(current_part);
    }

    Ok(parts)
}

fn parse_bitrate_bps(bitrate: &str) -> Result<u64> {
    let s = bitrate.trim().to_lowercase();
    if let Some(n) = s.strip_suffix('k') {
        let kbps: u64 = n.parse().context("Invalid bitrate")?;
        Ok(kbps * 1000)
    } else if let Some(n) = s.strip_suffix('m') {
        let mbps: u64 = n.parse().context("Invalid bitrate")?;
        Ok(mbps * 1_000_000)
    } else {
        s.parse::<u64>().context("Invalid bitrate")
    }
}

#[allow(clippy::too_many_arguments)]
async fn build_m4b(
    chapters: &[crate::types::Chapter],
    chapter_files: &[&PathBuf],
    output_path: &Path,
    work_dir: &Path,
    bitrate: &str,
    cover_path: Option<&Path>,
    part_idx: usize,
    label: &str,
    total_duration_s: f64,
) -> Result<()> {
    let concat_file = work_dir.join(format!("ffmpeg_concat_{}.txt", part_idx));
    let metadata_file = work_dir.join(format!("ffmpeg_metadata_{}.txt", part_idx));

    // Use the output filename (without extension) as the title in metadata
    let title = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Audiobook");

    create_concat_file(&concat_file, chapter_files)?;
    create_metadata_file_for_chapters(&metadata_file, title, chapters)?;

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&concat_file)
        .arg("-i")
        .arg(&metadata_file);

    let has_cover = cover_path.map(|p| p.exists()).unwrap_or(false);
    if has_cover {
        cmd.arg("-i").arg(cover_path.unwrap());
    }

    cmd.arg("-map_metadata")
        .arg("1")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg(bitrate);

    if has_cover {
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
        .arg("+faststart")
        .arg("-progress")
        .arg("pipe:1") // stream progress to stdout
        .arg("-nostats")
        .arg("-y");
    cmd.arg(output_path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let pb = ProgressBar::new(total_duration_s as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/white}] {percent}%")
            .expect("Invalid template")
            .progress_chars("█▓░"),
    );
    pb.set_message(label.to_string());

    let mut child = cmd.spawn().context("Failed to spawn ffmpeg")?;
    let stdout = child.stdout.take().context("Failed to capture ffmpeg stdout")?;

    for line in BufReader::new(stdout).lines() {
        let line = line.unwrap_or_default();
        // ffmpeg -progress emits "out_time_ms=<microseconds>"
        if let Some(val) = line.strip_prefix("out_time_ms=") {
            if let Ok(us) = val.trim().parse::<u64>() {
                let secs = us / 1_000_000;
                pb.set_position(secs.min(total_duration_s as u64));
            }
        }
    }

    let status = child.wait().context("Failed to wait for ffmpeg")?;

    let _ = std::fs::remove_file(&concat_file);
    let _ = std::fs::remove_file(&metadata_file);

    if !status.success() {
        pb.abandon_with_message(format!("❌ ffmpeg failed: {}", output_path.display()));
        anyhow::bail!("ffmpeg failed with status: {}", status);
    }

    pb.finish_with_message(format!("✅ {}", output_path.display()));
    Ok(())
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

fn create_concat_file<P: AsRef<Path>>(concat_file: &Path, chapter_files: &[P]) -> Result<()> {
    let mut content = String::new();
    for file in chapter_files {
        let abs_path = file
            .as_ref()
            .canonicalize()
            .context("Failed to get absolute path for chapter file")?;
        content.push_str(&format!("file '{}'\n", abs_path.display()));
    }
    std::fs::write(concat_file, content).context("Failed to write concat file")?;
    Ok(())
}

fn create_metadata_file_for_chapters(
    metadata_file: &Path,
    title: &str,
    chapters: &[crate::types::Chapter],
) -> Result<()> {
    let mut content = String::new();
    content.push_str(";FFMETADATA1\n");
    content.push_str(&format!("title={}\n", escape_metadata(title)));
    content.push_str("media_type=2\n");

    let mut current_time_ms: u64 = 0;
    for chapter in chapters {
        let duration_ms = (chapter.duration * 1000.0) as u64;
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
