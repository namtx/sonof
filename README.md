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
3. Downloads each chapter with real-time progress (⬇️)
4. Generates unique artwork for each chapter with the chapter number overlaid (🎨)
5. Embeds the artwork into each chapter file
6. Cleans up temporary files when finished
7. This allows you to easily identify chapters by their artwork in your audio player

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

## License

MIT

## Disclaimer

This tool is for personal use only. Please respect the terms of service of the Fonos platform and only download content you have legally purchased or have access to.

