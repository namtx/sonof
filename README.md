# Sonof

A command-line tool to download audiobooks from the Fonos app.

## Features

- 🔐 Authenticate with JWT token
- 📚 List all books in your library
- ℹ️ View detailed book information
- ⬇️ Download audiobook chapters
- 📊 Progress bar for downloads
- 🎯 Selective chapter downloads

## Installation

### Prerequisites

- Rust 1.70 or later

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
- Book ID (entity ID)
- Book entity type
- Purchase date

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
```

Example:
```bash
sonof download 1120 --output ~/Audiobooks/MyBook
```

#### Download specific chapters only

```bash
sonof download BOOK_ID --chapters "1,2,3,5,10"
```

This will only download chapters 1, 2, 3, 5, and 10.

Example:
```bash
sonof download 1120 --chapters "1,2,3" --output ~/Audiobooks
```

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
```

## File Structure

Downloaded files are organized as follows:

```
Book_Title/
├── 001_chapter_file.m4a
├── 002_chapter_file.m4a
├── 003_chapter_file.m4a
└── ...
```

Each chapter is prefixed with a three-digit number for proper ordering.

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

## License

MIT

## Disclaimer

This tool is for personal use only. Please respect the terms of service of the Fonos platform and only download content you have legally purchased or have access to.

