use anyhow::{Context, Result};
use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use std::path::{Path, PathBuf};
use std::process::Command;

const CDN_BASE_URL: &str = "https://cdn.fonos.dev";

/// Download the book cover image
pub async fn download_cover_image(cover_url: &str, output_dir: &Path) -> Result<PathBuf> {
    // Construct full CDN URL
    let cdn_url = if cover_url.starts_with("http") {
        cover_url.to_string()
    } else {
        format!("{}/{}", CDN_BASE_URL, cover_url.trim_start_matches('/'))
    };

    // Download the cover image
    let response = reqwest::get(&cdn_url)
        .await
        .context("Failed to download cover image")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to download cover image: {} - URL: {}",
            response.status(),
            cdn_url
        );
    }

    // Get content type to determine file extension
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let extension = match content_type.as_str() {
        ct if ct.contains("png") => "png",
        ct if ct.contains("webp") => "webp",
        ct if ct.contains("gif") => "gif",
        _ => "jpg",
    };

    let cover_path = output_dir.join(format!("cover.{}", extension));

    // Skip if already downloaded
    if cover_path.exists() {
        return Ok(cover_path);
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read cover image")?;

    if bytes.is_empty() {
        anyhow::bail!("Downloaded cover image is empty");
    }

    // Save the cover image
    std::fs::write(&cover_path, &bytes).context("Failed to save cover image")?;

    // Verify the image can be opened
    image::open(&cover_path).with_context(|| {
        format!(
            "Downloaded file is not a valid image format. Content-Type: {}",
            content_type
        )
    })?;

    Ok(cover_path)
}

/// Generate a chapter artwork image with chapter number overlay
pub fn generate_simple_chapter_artwork(
    cover_path: &Path,
    chapter_number: usize,
    output_dir: &Path,
) -> Result<PathBuf> {
    let output_path = output_dir.join(format!("chapter_{}_artwork.jpg", chapter_number));

    // Skip if already generated
    if output_path.exists() {
        return Ok(output_path);
    }

    // Check if cover image exists
    if !cover_path.exists() {
        anyhow::bail!("Cover image not found at path: {}", cover_path.display());
    }

    // Load the cover image
    let mut img = image::open(cover_path)
        .with_context(|| format!("Failed to open cover image at: {}", cover_path.display()))?
        .to_rgba8();

    let (width, height) = img.dimensions();

    // Calculate dimensions for the chapter number badge (centered)
    let badge_size = width.min(height) / 3; // Make it about 1/3 of the smaller dimension
    let badge_x = (width - badge_size) / 2;
    let badge_y = (height - badge_size) / 2;

    // Draw a semi-transparent dark background for the badge in the center
    let badge_rect = Rect::at(badge_x as i32, badge_y as i32).of_size(badge_size, badge_size);
    draw_filled_rect_mut(&mut img, badge_rect, Rgba([0u8, 0u8, 0u8, 220u8]));

    // Convert back to DynamicImage and save
    let dynamic_img = DynamicImage::ImageRgba8(img);
    dynamic_img
        .save(&output_path)
        .context("Failed to save chapter artwork")?;

    // Use ImageMagick to add text if available, otherwise just use the badge
    let _ = add_text_with_imagemagick(&output_path, chapter_number, width, height, badge_size);

    Ok(output_path)
}

/// Try to add text using ImageMagick if available
fn add_text_with_imagemagick(
    image_path: &Path,
    chapter_number: usize,
    _width: u32,
    _height: u32,
    badge_size: u32,
) -> Result<()> {
    // Use a large font size - about 60% of the badge size
    let font_size = (badge_size as f32 * 0.6) as u32;

    // Try 'magick' first (ImageMagick v7), then fall back to 'convert' (v6)
    let commands = ["magick", "convert"];

    for cmd in &commands {
        let mut command = Command::new(cmd);
        command
            .arg(image_path)
            .arg("-font")
            .arg("Helvetica-Bold")
            .arg("-pointsize")
            .arg(format!("{}", font_size))
            .arg("-fill")
            .arg("white")
            .arg("-gravity")
            .arg("center")
            .arg("-annotate")
            .arg("+0+0")
            .arg(format!("{}", chapter_number))
            .arg(image_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        if let Ok(status) = command.status() {
            if status.success() {
                return Ok(());
            }
        }
    }

    // ImageMagick not available or failed, that's ok
    Ok(())
}

/// Embed artwork into an audio file using ffmpeg
pub async fn embed_artwork(audio_file: &Path, artwork_file: &Path) -> Result<()> {
    let temp_output = audio_file.with_extension("tmp.m4a");

    // Use ffmpeg to embed the artwork
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(audio_file)
        .arg("-i")
        .arg(artwork_file)
        .arg("-map")
        .arg("0:a")
        .arg("-map")
        .arg("1:v")
        .arg("-c")
        .arg("copy")
        .arg("-disposition:v:0")
        .arg("attached_pic")
        .arg("-y")
        .arg(&temp_output)
        .output()
        .context("Failed to execute ffmpeg for artwork embedding")?;

    if !status.status.success() {
        // If ffmpeg fails, just keep the original file
        let _ = std::fs::remove_file(&temp_output);
        return Ok(());
    }

    // Replace original file with the one that has artwork
    std::fs::rename(&temp_output, audio_file)
        .context("Failed to replace audio file with artwork-embedded version")?;

    Ok(())
}

/// Generate chapter artwork and embed it into the audio file
pub async fn add_chapter_artwork(
    audio_file: &Path,
    cover_path: &Path,
    chapter_number: usize,
    output_dir: &Path,
) -> Result<()> {
    // Generate the chapter artwork
    let artwork_path = generate_simple_chapter_artwork(cover_path, chapter_number, output_dir)?;

    // Embed the artwork into the audio file
    embed_artwork(audio_file, &artwork_path).await?;

    Ok(())
}
