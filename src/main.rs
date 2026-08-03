mod api;
mod apple_music;
mod artwork;
mod audiobook;
mod chapters;
mod config;
mod types;

use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sonof")]
#[command(version)]
#[command(about = "A command line tool to download audiobooks from the Fonos app", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate and save JWT token
    Login {
        /// JWT token from Fonos app
        #[arg(short, long)]
        token: String,
    },
    /// List all books in your library
    List,
    /// Show details of a specific book
    Info {
        /// Book entity ID
        book_id: u32,
    },
    /// Download a book
    Download {
        /// Book entity ID
        book_id: u32,
        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Download specific chapters only. Supports singles and ranges,
        /// e.g. "1,2,3", "1..5", "2..", "..5", or "1,3..5,7.."
        #[arg(short, long)]
        chapters: Option<String>,
        /// Skip joining chapters into a single m4b audiobook file
        #[arg(long)]
        no_join: bool,
        /// Audio bitrate for compilation (default: 128k, examples: 64k, 96k, 128k, 192k, 256k)
        #[arg(short = 'b', long, default_value = "128k")]
        bitrate: String,
        /// Add downloaded chapters to Apple Music playlist (macOS only)
        #[arg(short = 'a', long)]
        add_to_apple_music: bool,
        /// Replace existing files and playlists if they already exist
        #[arg(short = 'R', long)]
        replace: bool,
    },
    /// Join already-downloaded chapters into m4b audiobook file(s) with chapter metadata.
    /// Files larger than 500 MB are automatically split into parts.
    Join {
        /// Book entity ID (used to fetch chapter metadata)
        book_id: u32,
        /// Directory containing the downloaded chapter files (*.m4a)
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory (default: same as input directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Audio bitrate for re-encoding (default: 128k)
        #[arg(short = 'b', long, default_value = "128k")]
        bitrate: String,
        /// Path to cover image to embed (optional)
        #[arg(short = 'c', long)]
        cover: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login { token } => {
            config::save_token(&token)?;
            println!("✓ Token saved successfully!");
        }
        Commands::List => {
            let token = config::load_token()?;
            let client = api::FonosClient::new(token);

            println!("Fetching your library...\n");
            let books = client.list_books().await?;

            if books.is_empty() {
                println!("No books found in your library.");
            } else {
                println!("Found {} book(s):\n", books.len());
                for book in books {
                    println!("  [{}] {}", book.book_id, book.title);
                }
            }
        }
        Commands::Info { book_id } => {
            let token = config::load_token()?;
            let client = api::FonosClient::new(token);

            let book = client.get_book_details(book_id).await?;

            println!("Title: {}", book.title);
            println!("Duration: {:.2} hours", book.duration / 3600.0);
            println!("Price: {}", book.price);
            println!("Status: {}", book.status);
            println!("Chapters: {}", book.chapters.len());

            if let Some(author) = &book.author {
                println!("Author: {}", serde_json::to_string_pretty(author)?);
            }

            if !book.categories.is_empty() {
                println!("\nCategories:");
                for cat in &book.categories {
                    println!("  - {}", cat.name);
                }
            }

            if !book.chapters.is_empty() {
                println!("\nChapters:");
                for (idx, chapter) in book.chapters.iter().enumerate() {
                    println!(
                        "  {}. {} ({:.2} min)",
                        idx + 1,
                        chapter.name,
                        chapter.duration / 60.0
                    );
                }
            }
        }
        Commands::Download {
            book_id,
            output,
            chapters,
            no_join,
            bitrate,
            add_to_apple_music,
            replace,
        } => {
            let token = config::load_token()?;
            let client = api::FonosClient::new(token);

            let book = client.get_book_details(book_id).await?;

            // Determine output directory
            let output_dir =
                output.unwrap_or_else(|| PathBuf::from(&sanitize_filename(&book.title)));

            // Handle replace flag
            if replace && output_dir.exists() {
                println!("🗑️  Removing existing directory: {}", output_dir.display());
                std::fs::remove_dir_all(&output_dir)?;
            }

            std::fs::create_dir_all(&output_dir)?;

            // Parse chapter filter (supports singles and ranges, e.g. "1,3..5,7..")
            let chapter_filter = chapters.as_deref().map(chapters::parse_chapters);

            // Get resource permissions for CDN access
            let resource_permissions_pb: ProgressBar = ProgressBar::new_spinner();
            resource_permissions_pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .expect("Invalid template")
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            resource_permissions_pb.set_message("📥 Fetching resource permissions...");
            let permissions = client.get_resource_permissions(book_id).await?;
            resource_permissions_pb.finish_with_message("✓ Resource permissions fetched");

            // Create temporary directory for cover and artwork generation
            let temp_dir = std::env::temp_dir().join(format!("sonof_{}", book_id));
            std::fs::create_dir_all(&temp_dir)?;

            // Download book cover for artwork generation
            let cover_pb = ProgressBar::new_spinner();
            cover_pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .expect("Invalid template")
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            cover_pb.set_message("📥 Downloading book cover...");
            cover_pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let cover_result =
                artwork::download_cover_image(&book.cover_image_url, &temp_dir).await;
            let cover_path = match cover_result {
                Ok(path) => {
                    cover_pb.finish_with_message("✓ Cover downloaded");

                    // Copy cover to output directory
                    if let Some(filename) = path.file_name() {
                        let output_cover_path = output_dir.join(filename);
                        if let Err(e) = std::fs::copy(&path, &output_cover_path) {
                            eprintln!("⚠ Warning: Failed to copy cover to output directory: {}", e);
                        }
                    }

                    Some(path)
                }
                Err(e) => {
                    cover_pb.finish_with_message(format!("⚠ Failed to download cover: {}", e));
                    None
                }
            };

            // Track downloaded chapter files for compilation
            let mut downloaded_files = Vec::new();

            // Determine which chapters to download
            let chapters_to_download: Vec<(usize, &_)> = book
                .chapters
                .iter()
                .enumerate()
                .filter(|(idx, _)| {
                    let chapter_num = (idx + 1) as u32;
                    chapter_filter.as_ref().is_none_or(|filter| {
                        filter.iter().any(|spec| spec.matches(chapter_num))
                    })
                })
                .collect();

            let total_chapters = chapters_to_download.len();

            // Create overall progress bar
            let overall_pb = ProgressBar::new(total_chapters as u64);
            overall_pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg} [{bar:40.green/white}] {pos}/{len}")
                    .expect("Invalid template")
                    .progress_chars("█▓░"),
            );
            overall_pb.set_message("📚 Starting download...");

            // Download chapters
            for (_completed, (idx, chapter)) in chapters_to_download.iter().enumerate() {
                let chapter_num = idx + 1;

                // Generate simple numeric filename (001.m4a, 002.m4a, etc.)
                let output_path = output_dir.join(format!("{:03}.m4a", chapter_num));

                // Check if chapter already exists
                if output_path.exists() {
                    overall_pb.set_message(format!(
                        "✓ Skipping Chapter {}/{} - {} (already exists)",
                        chapter_num, total_chapters, chapter.name
                    ));
                    downloaded_files.push(output_path.clone());

                    overall_pb.inc(1);
                    continue;
                }

                // Download
                overall_pb.set_message(format!(
                    "⬇️  Downloading Chapter {}/{} - {}",
                    chapter_num, total_chapters, chapter.name
                ));

                client
                    .download_chapter(&chapter.url, &permissions, &output_path)
                    .await?;

                // Generate and embed chapter artwork
                if let Some(ref cover) = cover_path {
                    overall_pb.set_message(format!(
                        "🎨 Generating artwork Chapter {}/{} - {}",
                        chapter_num, total_chapters, chapter.name
                    ));

                    // Generate the chapter artwork (saved in temp_dir)
                    match artwork::generate_simple_chapter_artwork(cover, chapter_num, &temp_dir) {
                        Ok(artwork_path) => {
                            // Embed artwork in file
                            if let Err(e) =
                                artwork::embed_artwork(&output_path, &artwork_path).await
                            {
                                overall_pb.println(format!("  ⚠ Failed to embed artwork: {}", e));
                            }
                        }
                        Err(e) => {
                            overall_pb.println(format!("  ⚠ Failed to generate artwork: {}", e));
                        }
                    }
                }

                downloaded_files.push(output_path.clone());

                overall_pb.inc(1);
            }

            overall_pb.finish_with_message("✅ All chapters complete!");

            // Join chapters into m4b unless explicitly skipped
            if !no_join && !downloaded_files.is_empty() {
                println!();
                let cover_for_m4b = cover_path.as_ref().map(|p| p.as_path());
                audiobook::compile_to_m4b_with_artwork(
                    &book,
                    &downloaded_files,
                    &output_dir,
                    &bitrate,
                    cover_for_m4b,
                )
                .await?;
            }

            // Add to Apple Music playlist if requested
            if add_to_apple_music && !downloaded_files.is_empty() {
                println!();
                println!("🎵 Apple Music Integration");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                // Add files to playlist (artwork is already embedded in the audio files)
                match apple_music::add_to_playlist(&book.title, &downloaded_files, replace) {
                    Ok(_) => {
                        println!(
                            "✅ Successfully added {} files to Apple Music playlist",
                            downloaded_files.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to add to Apple Music: {}", e);
                    }
                }
            }

            // Clean up temporary directory
            if temp_dir.exists() {
                let _ = std::fs::remove_dir_all(&temp_dir);
                println!("Cleaned up temporary files");
            }
        }
        Commands::Join {
            book_id,
            input,
            output,
            bitrate,
            cover,
        } => {
            let token = config::load_token()?;
            let client = api::FonosClient::new(token);

            let book = client.get_book_details(book_id).await?;

            // Collect chapter files from input directory, sorted by name
            let mut chapter_files: Vec<PathBuf> = std::fs::read_dir(&input)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|e| e == "m4a").unwrap_or(false))
                .collect();
            chapter_files.sort();

            if chapter_files.is_empty() {
                anyhow::bail!("No .m4a files found in: {}", input.display());
            }

            if chapter_files.len() != book.chapters.len() {
                eprintln!(
                    "⚠ Warning: found {} .m4a files but book has {} chapters — chapter names may be misaligned",
                    chapter_files.len(),
                    book.chapters.len()
                );
            }

            let output_dir = output.unwrap_or_else(|| input.clone());
            std::fs::create_dir_all(&output_dir)?;

            println!(
                "📦 Joining {} chapters from {} ...",
                chapter_files.len(),
                input.display()
            );

            let cover_path = cover.as_ref().map(|p| p.as_path());
            audiobook::compile_to_m4b_with_artwork(
                &book,
                &chapter_files,
                &output_dir,
                &bitrate,
                cover_path,
            )
            .await?;
        }
    }

    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}
