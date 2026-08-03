# Quick Start Guide

Get started with Sonof in 5 minutes!

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd sonof

# Build the release binary
cargo build --release

# Optional: Add to PATH
sudo cp target/release/sonof /usr/local/bin/
```

## Getting Your JWT Token

Before using Sonof, you need to get your JWT token from the Fonos app:

### Method 1: Using Browser Developer Tools (Desktop)

1. Open Fonos in your web browser: https://fonos.vn
2. Log in to your account
3. Press `F12` (Windows/Linux) or `Cmd+Option+I` (Mac) to open Developer Tools
4. Go to the **Network** tab
5. Refresh the page or navigate to your library
6. Look for any request to `production.fonos.dev`
7. Click on the request and go to the **Headers** tab
8. Find the `Authorization` header
9. Copy the token (it looks like: `Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`)
10. You can use the token with or without the `Bearer ` prefix

### Method 2: Using Mobile App (Advanced)

1. Install a network proxy tool like Charles Proxy or mitmproxy
2. Configure your phone to use the proxy
3. Open the Fonos app
4. Inspect the network traffic to find the `Authorization` header

## Basic Usage

### 1. Login

Save your JWT token:

```bash
sonof login --token "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Or with Bearer prefix:

```bash
sonof login --token "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### 2. List Your Books

```bash
sonof list
```

Example output:
```
Fetching your library...

Found 3 book(s):

  [1120] english_book - 2024-03-31T17:57:42.549Z
  [1184] english_book - 2024-07-15T10:30:00.000Z
  [935] english_book - 2024-01-20T08:15:30.000Z
```

### 3. Get Book Information

```bash
sonof info 1120
```

Example output:
```
Fetching book details...

Title: Pháo Đài Số
Duration: 16.29 hours
Price: 299000
Status: Published
Chapters: 129

Categories:
  - Văn học
  - Tiểu thuyết Giả tưởng

Chapters:
  1. CHƯƠNG 1 (11.04 min)
  2. CHƯƠNG 2 (1.45 min)
  3. CHƯƠNG 3 (35.95 min)
  ...
```

### 4. Download a Book

Download all chapters:

```bash
sonof download 1120
```

Download to specific directory:

```bash
sonof download 1120 --output ~/Audiobooks/PhaoDaiSo
```

Download specific chapters only:

```bash
sonof download 1120 --chapters "1,2,3,10,20"
```

Chapter ranges are supported too (`..` is inclusive, open ends allowed):

```bash
# Chapters 1 through 10
sonof download 1120 --chapters "1..10"

# From chapter 3 to the end
sonof download 1120 --chapters "3.."

# First 5 chapters
sonof download 1120 --chapters "..5"
```

## Common Commands

```bash
# Show help
sonof --help

# Show version
sonof --version

# Help for specific command
sonof download --help

# List books
sonof list

# Download book #1120 to current directory
sonof download 1120

# Download chapters 1-10 to custom directory
sonof download 1120 --chapters "1..10" --output ~/Books
```

## Tips & Tricks

### 1. Downloading Multiple Books

```bash
# Create a script
for book_id in 1120 1184 935; do
    sonof download $book_id --output ~/Audiobooks/book_$book_id
done
```

### 2. Selective Chapter Downloads

```bash
# Download first 5 chapters only
sonof download 1120 --chapters "1..5"

# Download specific chapters
sonof download 1120 --chapters "1,10,20,30"
```

### 3. Organizing Downloads

```bash
# Create organized structure
mkdir -p ~/Audiobooks/Fiction
sonof download 1120 --output ~/Audiobooks/Fiction/PhaoDaiSo
```

## Troubleshooting

### Token Expired

If you get `API error: 401` or `API error: 403`:

```bash
# Get a new token from Fonos website
# Then login again
sonof login --token "YOUR_NEW_TOKEN"
```

### Download Interrupted

Simply run the download command again. The tool will re-download the chapters.

### File Already Exists

The tool will overwrite existing files. Make sure to use different output directories if you want to keep multiple versions.

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check [CONTRIBUTING.md](CONTRIBUTING.md) if you want to contribute
- Review [specs/specs.md](specs/specs.md) for API documentation

## Support

If you encounter any issues:
1. Check that your token is valid and not expired
2. Verify you have purchased/have access to the book
3. Check your internet connection
4. Open an issue on GitHub with details about the error

Enjoy downloading your audiobooks! 🎧

