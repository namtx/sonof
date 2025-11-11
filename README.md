# Sonof

A command-line tool to download audiobooks from the Fonos app.

## Features

- 🔐 Authenticate with JWT token
- 📚 List all books in your library with titles
- ℹ️ View detailed book information (title, author, duration, chapters, categories)
- ⬇️ Download audiobook chapters with progress indicators
- 📊 Real-time progress bars with emoji feedback
- 🎯 Selective chapter downloads (choose specific chapters)
- 🎨 Automatic chapter artwork generation with chapter numbers overlaid
- 📖 Compile chapters into single m4b audiobook file with embedded artwork
- 🔧 Customizable audio quality (bitrate) for compiled audiobooks
- 🎵 Add downloaded chapters to Apple Music playlist (macOS only)
- 🔄 Replace mode to overwrite existing files and playlists
- ⏯️ Smart resume: automatically skip already downloaded chapters
- 🧹 Automatic cleanup of temporary files

## Installation

### Prerequisites

- Rust 1.70 or later
- FFmpeg (required for artwork embedding and compiling chapters into m4b files)
- ImageMagick (optional, for better chapter number rendering on artwork)

### Build from source

```bash
git clone <repository-url>
cd sonof
cargo build --release
```

The binary will be available at `target/release/sonof`.

## Usage

### 1. Authentication

First, you need to authenticate with your Fonos JWT token:

```bash
sonof login --token YOUR_JWT_TOKEN
```

The token will be saved in `~/.sonof/token` for future use.

#### How to get your JWT token

1. Open the Fonos app in your browser
2. Open Developer Tools (F12)
3. Go to the Network tab
4. Make any request to the Fonos API
5. Look for the `Authorization` header in the request
6. Copy the token (it starts with `Bearer ` - you can include or exclude the "Bearer " part)

### 2. List your books

View all books in your library:

```bash
sonof list
```

This will display:
- Book ID
- Book title

### 3. View book details

Get detailed information about a specific book:

```bash
sonof info BOOK_ID
```

Example:
```bash
sonof info 1120
```

This will show:
- Title
- Author (if available)
- Duration
- Price
- Status
- Number of chapters
- Categories
- Chapter list with durations

### 4. Download a book

Download all chapters of a book:

```bash
sonof download BOOK_ID
```

Example:
```bash
sonof download 1120
```

#### Download to a specific directory

```bash
sonof download BOOK_ID --output /path/to/directory
# or use the short flag
sonof download BOOK_ID -o /path/to/directory
```

Example:
```bash
sonof download 1120 --output ~/Audiobooks/MyBook
# or
sonof download 1120 -o ~/Audiobooks/MyBook
```

**Note**: When downloading chapters, the tool automatically:
1. Fetches resource permissions for CDN access (📥)
2. Downloads the book's cover image (📥)
3. Checks if each chapter already exists and skips it if present (✓)
4. Downloads each chapter with real-time progress (⬇️)
5. Generates unique artwork for each chapter with the chapter number overlaid (🎨)
6. Embeds the artwork into each chapter file
7. Cleans up temporary files when finished
8. This allows you to easily identify chapters by their artwork in your audio player

**Smart Resume**: If a download is interrupted or fails, simply run the same command again. Already downloaded chapters will be automatically skipped, and only missing chapters will be downloaded. Use the `--replace` (`-R`) flag if you want to force re-download all chapters.

#### Download specific chapters only

```bash
sonof download BOOK_ID --chapters "1,2,3,5,10"
# or use the short flag
sonof download BOOK_ID -c "1,2,3,5,10"
```

This will only download chapters 1, 2, 3, 5, and 10.

Example:
```bash
sonof download 1120 --chapters "1,2,3" --output ~/Audiobooks
# or with short flags
sonof download 1120 -c "1,2,3" -o ~/Audiobooks
```

#### Compile chapters into a single m4b audiobook

```bash
sonof download BOOK_ID --compile
# or use the short flag
sonof download BOOK_ID -m
```

This will download all chapters and automatically compile them into a single `.m4b` audiobook file with:
- Embedded chapter markers
- Book metadata (title, author, description, genre)
- Book cover artwork embedded
- Proper audiobook format
- AAC encoding for maximum compatibility

Individual chapter files will also have chapter-specific artwork with the chapter number displayed.

The compilation process (📦):
- Shows a real-time progress spinner
- Automatically re-encodes audio to AAC format for m4b compatibility
- Embeds chapter markers and metadata
- Embeds the book's cover artwork

Example:
```bash
sonof download 1120 --compile --output ~/Audiobooks
```

**Customize audio quality:**

```bash
sonof download BOOK_ID --compile --bitrate 192k
# or use the short flag
sonof download BOOK_ID -m -b 192k
```

Available bitrate options:
- `64k` - Lower quality, smaller file size
- `96k` - Good quality for speech
- `128k` - **Default**, excellent quality for audiobooks
- `192k` - High quality
- `256k` - Very high quality, larger file size

#### Add chapters to Apple Music playlist with artwork

```bash
sonof download BOOK_ID --add-to-apple-music
# or use the short flag
sonof download BOOK_ID -a
```

This will automatically:
1. Add all downloaded chapter files to an Apple Music playlist named after the book title
2. Set the chapter-specific artwork for each track directly in Apple Music using AppleScript
3. The artwork is set via Apple Music's API, ensuring maximum compatibility

The playlist will be created if it doesn't exist, or chapters will be added to the existing playlist.

**How it works (Batch Processing):**
1. **Phase 1**: Add all files to the playlist quickly (no delays)
2. **Phase 2**: Wait 3 seconds for Apple Music to index all the files
3. **Phase 3**: Set artwork for all tracks in batch (with minimal retries if needed)

This 3-phase approach is much more efficient than setting artwork immediately after each file:
- Faster overall process (no waiting between individual files)
- Higher success rate (Apple Music has time to index)
- Better user experience with clear progress indicators
- Each chapter file also has artwork embedded via ffmpeg (for compatibility with other players)
- Both PNG and JPEG artwork formats are supported

**Note**: This feature is only available on macOS and requires the Music app to be installed.

Example:
```bash
sonof download 1120 --add-to-apple-music
# or with other options
sonof download 1120 -a -o ~/Audiobooks
# or combine with compilation
sonof download 1120 -a -m
```

#### Replace existing files and playlists

```bash
sonof download BOOK_ID --replace
# or use the short flag
sonof download BOOK_ID -R
```

When using the `--replace` flag:
- If the output directory already exists, it will be **completely removed** and recreated
- If an Apple Music playlist with the book title exists, it will be **deleted and recreated**
- This is useful when you want to re-download a book or refresh your library

**⚠️ Warning**: The replace option will delete all existing files in the output directory and remove the playlist. Use with caution!

Example:
```bash
# Re-download and replace all files
sonof download 1120 --replace

# Replace files and playlist
sonof download 1120 -R -a

# Combine with all other options
sonof download 1120 -R -a -m -b 192k
```

**Note**: FFmpeg must be installed for this feature to work. The compilation process re-encodes audio to AAC format for m4b compatibility.

Installing FFmpeg (required):
- **macOS**: `brew install ffmpeg`
- **Linux**: `sudo apt install ffmpeg` (Debian/Ubuntu) or `sudo yum install ffmpeg` (CentOS/RHEL)
- **Windows**: Download from [ffmpeg.org](https://ffmpeg.org/)

Installing ImageMagick (optional, for enhanced chapter number rendering):
- **macOS**: `brew install imagemagick`
- **Linux**: `sudo apt install imagemagick` (Debian/Ubuntu) or `sudo yum install imagemagick` (CentOS/RHEL)
- **Windows**: Download from [imagemagick.org](https://imagemagick.org/)

## Command-Line Flags Reference

### Login Command
```bash
sonof login --token <TOKEN>
```
| Flag | Short | Description |
|------|-------|-------------|
| `--token` | `-t` | JWT token from Fonos app |

### List Command
```bash
sonof list
```
No additional flags required.

### Info Command
```bash
sonof info <BOOK_ID>
```
No additional flags required.

### Download Command
```bash
sonof download <BOOK_ID> [OPTIONS]
```
| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--output` | `-o` | Output directory for downloaded files | Current directory / Book title |
| `--chapters` | `-c` | Comma-separated list of chapter numbers to download | All chapters |
| `--compile` | `-m` | Compile chapters into a single m4b audiobook | false |
| `--bitrate` | `-b` | Audio bitrate for compilation (e.g., 64k, 96k, 128k, 192k, 256k) | 128k |
| `--add-to-apple-music` | `-a` | Add downloaded chapters to Apple Music playlist (macOS only) | false |
| `--replace` | `-R` | Replace existing files and playlists (⚠️ deletes existing content) | false |

## Examples

### Complete workflow

```bash
# 1. Login
sonof login --token "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 2. List your books
sonof list

# 3. Get info about a specific book
sonof info 1120

# 4. Download the book
sonof download 1120

# 5. Download only specific chapters
sonof download 1120 --chapters "1,2,3"
# or using short flag
sonof download 1120 -c "1,2,3"

# 6. Download and compile into a single m4b audiobook
sonof download 1120 --compile
# or using short flags
sonof download 1120 -m

# 7. Download and compile with higher quality
sonof download 1120 --compile --bitrate 192k
# or using short flags
sonof download 1120 -m -b 192k

# 8. Download and add to Apple Music playlist (macOS only)
sonof download 1120 --add-to-apple-music
# or using short flag
sonof download 1120 -a

# 9. Download, compile, and add to Apple Music in one command
sonof download 1120 --compile --add-to-apple-music
# or using short flags
sonof download 1120 -m -a

# 10. Replace existing download and playlist
sonof download 1120 --replace
# or using short flag
sonof download 1120 -R

# 11. Replace everything with all options combined
sonof download 1120 --replace --compile --add-to-apple-music --bitrate 192k
# or using short flags
sonof download 1120 -R -m -a -b 192k
```

## File Structure

Downloaded files are organized as follows:

### Without --compile flag:
```
Book_Title/
├── cover.jpg                      (book cover image)
├── chapter_1_artwork.jpg          (chapter 1 artwork with number)
├── chapter_2_artwork.jpg          (chapter 2 artwork with number)
├── chapter_3_artwork.jpg          (chapter 3 artwork with number)
├── 001_chapter_file.m4a           (with embedded chapter 1 artwork)
├── 002_chapter_file.m4a           (with embedded chapter 2 artwork)
├── 003_chapter_file.m4a           (with embedded chapter 3 artwork)
└── ...
```

Each chapter is:
- Prefixed with a three-digit number for proper ordering
- Embedded with unique artwork showing the chapter number

### With --compile flag:
```
Book_Title/
├── cover.jpg                      (book cover image)
├── chapter_1_artwork.jpg          (chapter 1 artwork with number)
├── chapter_2_artwork.jpg          (chapter 2 artwork with number)
├── 001_chapter_file.m4a           (with embedded chapter 1 artwork)
├── 002_chapter_file.m4a           (with embedded chapter 2 artwork)
├── ...
└── Book_Title.m4b                 (compiled audiobook with book cover)
```

The compiled `.m4b` file includes:
- All chapters with embedded chapter markers and metadata
- Book cover artwork (without chapter numbers)
- Individual chapter files retain their chapter-specific artwork

**Note**: The tool automatically cleans up temporary files used during artwork generation after the download completes.

## API Endpoints Used

- `GET /users/my-library` - List books in user's library
- `GET /books/{entityId}` - Get book details
- `GET /resource-permissions` - Get CDN access tokens
- CDN downloads with signed cookies and clearKey authentication

## Configuration

Configuration and tokens are stored in:
- **macOS/Linux**: `~/.sonof/`
- **Windows**: `%USERPROFILE%\.sonof\`

## Troubleshooting

### "No token found" error

Run `sonof login --token YOUR_TOKEN` first to authenticate.

### "API error: 401" or "API error: 403"

Your token might have expired. Get a new token from the Fonos app and login again.

### Download fails

- Check your internet connection
- Verify the book ID is correct
- Try getting fresh resource permissions by running the download command again

### "Failed to check ffmpeg" error when using --compile

FFmpeg is not installed or not in your PATH. Install it using:
- **macOS**: `brew install ffmpeg`
- **Linux**: `sudo apt install ffmpeg` (Debian/Ubuntu)
- **Windows**: Download from [ffmpeg.org](https://ffmpeg.org/) and add to PATH

### Compilation fails

- Ensure all chapter files were downloaded successfully
- Check that you have write permissions in the output directory
- Verify that the chapter files are valid audio files
- Try running the download again without the `--compile` flag first to verify the downloads work

### "codec not currently supported in container" error

This error has been fixed. The tool now automatically re-encodes audio to AAC format, which is compatible with m4b files. If you still encounter this issue:
- Make sure you're using the latest version
- Try using a different bitrate (e.g., `--bitrate 96k`)
- Verify that FFmpeg is properly installed and up to date

### Apple Music integration issues

If you encounter issues when using `--add-to-apple-music`:

**"Apple Music integration is only available on macOS"**
- This feature only works on macOS systems with the Music app installed
- On other operating systems, download the files without this flag

**"Music app is not available on this system"**
- Ensure the Music app is installed on your Mac
- Try opening Music app manually to verify it works

**"Failed to add to Apple Music" errors**
- Make sure the Music app has the necessary permissions
- Check that the downloaded files are in a location accessible to the Music app
- Try adding one file manually to verify Music app is working correctly
- If the playlist name contains special characters, the tool will handle escaping automatically

**Playlist already exists**
- By default, if a playlist with the book title already exists, the tool will add the chapters to that existing playlist
- Use the `--replace` flag if you want to delete and recreate the playlist instead
- This allows you to re-download or add additional chapters without creating duplicate playlists

### Artwork not showing in Apple Music

**New AppleScript Integration (v0.1.0+)**

As of the latest version, artwork is set directly in Apple Music using AppleScript when you use the `--add-to-apple-music` (`-a`) flag. This provides the most reliable artwork display in Apple Music.

**How the new system works:**
1. Artwork is embedded in the audio file via ffmpeg (for other players)
2. When adding to Apple Music, artwork is also set directly via AppleScript
3. This dual approach ensures artwork works in Apple Music and other audio players

**If artwork still isn't showing:**

**Batch processing with smart indexing**
- The tool uses a 3-phase approach: add files → wait for indexing → set artwork
- A 3-second initial wait allows Apple Music to index all files
- Minimal retries are needed (usually 0-3 per file) since files are already indexed
- You'll see clear progress: "Adding files...", "Waiting for indexing...", "Setting artwork..."
- Final summary shows success rate (e.g., "Set artwork for 30/31 tracks (1 failed)")

**Check Apple Music permissions**
- Go to System Settings > Privacy & Security > Automation
- Ensure your Terminal/iTerm has permission to control Music

**Verify the track was added**
```bash
# After running with -a flag, check in Apple Music:
1. Open the playlist (named after your book title)
2. Right-click on a track → "Song Info" or "Get Info"
3. Go to the "Artwork" tab
4. The chapter artwork should be displayed
```

**Force reimport**
- Remove the playlist from Apple Music
- Run the download command again with `-R -a` flags to replace everything
- This ensures fresh files and re-runs the AppleScript artwork setting

**If some artwork still fails**
With batch processing, failures are rare but can happen if:
- Your Mac is very slow or under heavy load
- You have a very large library that takes time to index
- The files are on a network drive with slow access

What to do:
1. Check the final summary (e.g., "Set artwork for 30/31 tracks (1 failed)")
2. For any failed tracks, the tool shows the artwork file path
3. Manually add artwork:
   - Open Apple Music
   - Find the track in the playlist
   - Right-click → "Get Info" → "Artwork" tab
   - Click "Add Artwork..." and select the `chapter_X_artwork.jpg` file shown in the error

**Tip**: If many tracks fail (more than 20%), try:
- Closing and reopening Apple Music before running the command
- Running the command again with `-R -a` to force fresh import
- Checking that your system isn't under heavy load

**Check embedded artwork (for other players)**
```bash
# Verify artwork is embedded in the file for non-Apple Music players
ffmpeg -i "path/to/chapter_file.m4a" 2>&1 | grep -i "video\|attached"
```
- You should see a line like "Video: mjpeg" or "attached_pic"
- This is for compatibility with other audio players

**Manual fallback**
- The tool saves `cover.jpg` and `chapter_X_artwork.jpg` files in the output directory
- You can manually add these to tracks in Apple Music via "Get Info" > "Artwork" > "Add Artwork" > "Add Artwork..." button

### Files already exist

If you try to download to a directory that already exists:
- **Without `--replace`**: Existing chapter files will be automatically skipped (smart resume)
- **With `--replace` (`-R`)**: The entire directory will be removed first, then all chapters re-downloaded

**Smart Resume Behavior**:
```bash
# First download - downloads chapters 1-10
sonof download 1120

# Download interrupted at chapter 5 - run the same command again
sonof download 1120
# Result: Skips chapters 1-4, continues from chapter 5

# Force re-download everything
sonof download 1120 -R
# Result: Removes all files and downloads everything fresh
```

Use `--replace` when:
- You want to ensure a clean, fresh download
- You suspect downloaded files are corrupted
- You want to remove old files that might not be part of the current download
- You want to reset both files and Apple Music playlists

## License

MIT

## Disclaimer

This tool is for personal use only. Please respect the terms of service of the Fonos platform and only download content you have legally purchased or have access to.

