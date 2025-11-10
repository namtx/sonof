# Contributing to Sonof

Thank you for your interest in contributing to Sonof! This document provides information about the project structure and development workflow.

## Project Structure

```
sonof/
├── src/
│   ├── main.rs      # CLI entry point and command handling
│   ├── api.rs       # Fonos API client implementation
│   ├── types.rs     # Data structures for API responses
│   └── config.rs    # Configuration and token management
├── specs/
│   └── specs.md     # API specifications and documentation
├── Cargo.toml       # Project dependencies
└── README.md        # User documentation
```

## Module Overview

### `main.rs`
- Defines the CLI structure using `clap`
- Handles command parsing and routing
- Implements the main application logic for each command

### `api.rs`
- Implements the `FonosClient` struct
- Handles all HTTP communication with Fonos API and CDN
- Manages authentication headers and download progress

### `types.rs`
- Defines all data structures used for API responses
- Uses `serde` for JSON serialization/deserialization

### `config.rs`
- Manages user configuration
- Handles JWT token storage in `~/.sonof/token`

## Development Setup

1. Install Rust (1.70 or later):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone the repository:
   ```bash
   git clone <repository-url>
   cd sonof
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run tests (when available):
   ```bash
   cargo test
   ```

5. Run the CLI:
   ```bash
   cargo run -- --help
   ```

## Code Style

This project follows standard Rust conventions:
- Use `rustfmt` for code formatting:
  ```bash
  cargo fmt
  ```

- Use `clippy` for linting:
  ```bash
  cargo clippy
  ```

## Adding New Features

### Adding a new CLI command

1. Add the command to the `Commands` enum in `main.rs`
2. Implement the command handler in the `match` statement
3. Add any necessary API methods to `api.rs`
4. Define response types in `types.rs` if needed
5. Update the README with usage examples

### Adding a new API endpoint

1. Define the response type in `types.rs`
2. Add a method to the `FonosClient` in `api.rs`
3. Use the method in the appropriate command handler in `main.rs`

## API Documentation

See `specs/specs.md` for detailed API specifications including:
- Endpoint URLs
- Request/response formats
- Authentication requirements
- CDN access patterns

## Testing

### Manual Testing

1. Obtain a valid JWT token from the Fonos app
2. Test the login command:
   ```bash
   cargo run -- login --token "YOUR_TOKEN"
   ```
3. Test listing books:
   ```bash
   cargo run -- list
   ```
4. Test getting book info:
   ```bash
   cargo run -- info BOOK_ID
   ```

### Future Improvements

- Add unit tests for utility functions
- Add integration tests for API client
- Mock API responses for testing
- Add CI/CD pipeline

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Test your changes thoroughly
6. Commit with clear messages: `git commit -m "Add feature X"`
7. Push to your fork: `git push origin feature/my-feature`
8. Open a Pull Request

## Common Issues

### Build Errors

If you encounter build errors:
1. Ensure you have the latest Rust version: `rustup update`
2. Clean the build: `cargo clean`
3. Rebuild: `cargo build`

### Runtime Errors

- **"No token found"**: Run the login command first
- **API 401/403**: Token expired, get a new one
- **Download fails**: Check network connection and token validity

## Questions?

Feel free to open an issue for:
- Bug reports
- Feature requests
- Questions about the codebase
- Documentation improvements

## License

By contributing to Sonof, you agree that your contributions will be licensed under the MIT License.

