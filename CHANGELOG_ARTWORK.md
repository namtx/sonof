# Chapter Artwork Feature Implementation

## Overview
Added support for automatically generating and embedding chapter numbers as artwork for each downloaded audiobook chapter.

## Features Implemented

### 1. Cover Image Download
- Automatically downloads the book's cover image from the Fonos CDN
- Caches the cover locally to avoid re-downloading
- Handles both full URLs and relative paths

### 2. Chapter Artwork Generation
- Creates unique artwork for each chapter based on the book cover
- Adds a semi-transparent dark badge at the bottom of the cover
- Overlays the chapter number on the badge using ImageMagick (if available)
- Falls back to badge-only artwork if ImageMagick is not installed
- Generated artwork files are saved as `chapter_{N}_artwork.jpg`

### 3. Artwork Embedding
- Automatically embeds the generated artwork into each downloaded chapter file
- Uses ffmpeg to embed artwork as an attached picture
- Preserves the original audio quality
- Non-intrusive: if embedding fails, the original file is kept

### 4. Compiled m4b Artwork
- When using the `--compile` flag, the final m4b file includes the book's original cover art
- Individual chapter files retain their chapter-specific artwork with numbers
- Provides visual distinction between the complete audiobook and individual chapters

## Technical Implementation

### New Files
- **src/artwork.rs**: New module handling all artwork-related functionality
  - `download_cover_image()`: Downloads book cover from CDN
  - `generate_simple_chapter_artwork()`: Creates chapter artwork with badge
  - `add_text_with_imagemagick()`: Adds chapter number text using ImageMagick
  - `embed_artwork()`: Embeds artwork into audio files using ffmpeg
  - `add_chapter_artwork()`: Main function orchestrating artwork generation and embedding

### Modified Files
- **Cargo.toml**: Added image processing dependencies
  - `image = "0.24"`
  - `imageproc = "0.23"`
  - `rusttype = "0.9"`
  - `ab_glyph = "0.2"`

- **src/main.rs**: Integrated artwork functionality into download flow
  - Registers the artwork module
  - Downloads cover image before downloading chapters
  - Embeds artwork into each chapter after download
  - Passes cover to m4b compilation

- **src/audiobook.rs**: Enhanced m4b compilation with artwork support
  - Added `compile_to_m4b_with_artwork()` function
  - Embeds cover art into compiled m4b file
  - Maintains backward compatibility with original `compile_to_m4b()`

- **README.md**: Updated documentation
  - Added artwork feature to features list
  - Documented ImageMagick as optional dependency
  - Updated file structure examples
  - Added notes about automatic artwork generation

- **specs/specs.md**: Updated specifications
  - Added artwork-related features to feature list

## Dependencies

### Required
- **FFmpeg**: Required for embedding artwork into audio files and compiling m4b

### Optional
- **ImageMagick**: Optional but recommended for rendering chapter numbers on artwork
  - If not available, artwork will have a dark badge without the chapter number text
  - Install with: `brew install imagemagick` (macOS), `sudo apt install imagemagick` (Linux)

## User Experience

### Download Flow
1. User runs: `sonof download BOOK_ID`
2. Tool downloads book cover image
3. For each chapter:
   - Downloads the chapter audio file
   - Generates chapter-specific artwork with chapter number
   - Embeds the artwork into the audio file
   - Shows progress: "Adding chapter artwork..." and "✓ Artwork added"

### File Output
```
Book_Title/
├── cover.jpg                      (original book cover)
├── chapter_1_artwork.jpg          (cover with chapter 1 badge)
├── chapter_2_artwork.jpg          (cover with chapter 2 badge)
├── 001_chapter_file.m4a           (with embedded chapter 1 artwork)
├── 002_chapter_file.m4a           (with embedded chapter 2 artwork)
└── ...
```

### With Compilation
```
Book_Title/
├── cover.jpg                      (original book cover)
├── chapter_*_artwork.jpg files...
├── 001_chapter_file.m4a           (with chapter artwork)
├── 002_chapter_file.m4a           (with chapter artwork)
└── Book_Title.m4b                 (with original cover artwork)
```

## Error Handling
- Gracefully handles missing cover images (continues without artwork)
- Non-fatal errors when artwork generation fails
- Preserves original audio files if embedding fails
- Informative error messages: "⚠ Failed to download cover: [reason]"

## Performance Considerations
- Artwork is only generated once per chapter (cached)
- Cover image is downloaded once and reused
- Minimal overhead added to download time
- Artwork generation is fast (< 1 second per chapter)

## Testing Recommendations
1. Test with a book that has a cover image
2. Test with ImageMagick installed and not installed
3. Test with `--compile` flag
4. Test with `--chapters` flag for selective downloads
5. Verify artwork displays correctly in audio players

## Future Enhancements
- Add option to disable artwork generation (--no-artwork flag)
- Support custom artwork templates
- Add chapter name in addition to chapter number
- Support different badge positions (top, bottom, overlay styles)
- Generate artwork for compiled m4b with total chapter count

