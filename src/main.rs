mod api;
mod artwork;
mod audiobook;
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
        /// Download specific chapters only (comma-separated, e.g., "1,2,3")
        #[arg(short, long)]
        chapters: Option<String>,
        /// Compile chapters into a single m4b audiobook file
        #[arg(short = 'm', long)]
        compile: bool,
        /// Audio bitrate for compilation (default: 128k, examples: 64k, 96k, 128k, 192k, 256k)
        #[arg(short = 'b', long, default_value = "128k")]
        bitrate: String,
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
            compile,
            bitrate,
        } => {
            let token = config::load_token()?;
            let client = api::FonosClient::new(token);

            let book = client.get_book_details(book_id).await?;

            // Determine output directory
            let output_dir =
                output.unwrap_or_else(|| PathBuf::from(&sanitize_filename(&book.title)));

            std::fs::create_dir_all(&output_dir)?;

            // Parse chapter filter
            let chapter_filter: Option<Vec<usize>> = chapters.map(|s| {
                s.split(',')
                    .filter_map(|n| n.trim().parse::<usize>().ok())
                    .collect()
            });

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
                    let chapter_num = idx + 1;
                    if let Some(ref filter) = chapter_filter {
                        filter.contains(&chapter_num)
                    } else {
                        true
                    }
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

                // Extract filename from URL
                let default_filename = format!("chapter_{}.m4a", chapter_num);
                let filename = chapter.url.split('/').last().unwrap_or(&default_filename);
                let output_path = output_dir.join(format!("{:03}_{}", chapter_num, filename));

                // Download
                overall_pb.set_message(format!(
                    "⬇️  Downloading Chapter {}/{} - {}",
                    chapter_num, total_chapters, chapter.name
                ));

                client
                    .download_chapter(&chapter.url, &permissions, &output_path)
                    .await?;

                // Add chapter artwork if cover was downloaded
                if let Some(ref cover) = cover_path {
                    overall_pb.set_message(format!(
                        "🎨 Artwork Chapter {}/{} - {}",
                        chapter_num, total_chapters, chapter.name
                    ));

                    if let Err(e) =
                        artwork::add_chapter_artwork(&output_path, cover, chapter_num, &temp_dir)
                            .await
                    {
                        overall_pb.println(format!("  ⚠ Failed to add artwork: {}", e));
                    }
                }

                downloaded_files.push(output_path);
                overall_pb.inc(1);
            }

            overall_pb.finish_with_message("✅ All chapters complete!");

            // Compile chapters into m4b if requested
            if compile && !downloaded_files.is_empty() {
                println!();
                let compile_pb = ProgressBar::new_spinner();
                compile_pb.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.green} {msg}")
                        .expect("Invalid template")
                        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
                );
                compile_pb.set_message("📦 Compiling chapters into m4b audiobook...");
                compile_pb.enable_steady_tick(std::time::Duration::from_millis(100));

                let cover_for_m4b = cover_path.as_ref().map(|p| p.as_path());
                let m4b_path = audiobook::compile_to_m4b_with_artwork(
                    &book,
                    &downloaded_files,
                    &output_dir,
                    &bitrate,
                    cover_for_m4b,
                )
                .await?;

                compile_pb.finish_with_message(format!("✓ Compiled to: {}", m4b_path.display()));
            }

            // Clean up temporary directory
            if temp_dir.exists() {
                let _ = std::fs::remove_dir_all(&temp_dir);
                println!("Cleaned up temporary files");
            }
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
