# rnm-rs

A simple Node.js version manager, written in Rust.

## Features

- Easily install and manage multiple Node.js versions
- Fast and lightweight, written in Rust
- Cross-platform support (Windows, macOS, Linux)

## Installation

### Binary Downloads

Prebuilt binaries are available for Windows, macOS, and Linux from the [GitHub Releases](https://github.com/zhuima/rnm-rs/releases) page.

### Building from Source

```bash
# Clone the repository
git clone https://github.com/zhuima/rnm-rs.git
cd rnm-rs

# Build the project
cargo build --release

# The binary will be available at target/release/rnm-rs
```

## Usage

```bash
# List available Node.js versions
rnm-rs list

# Install a specific Node.js version
rnm-rs install 16.14.0

# Use a specific Node.js version
rnm-rs use 16.14.0

# Show current Node.js version
rnm-rs current
```

## CI/CD

This project uses GitHub Actions for continuous integration and deployment:

- Builds are automatically triggered on pushes to the main branch and pull requests
- Release builds are created for Windows, macOS (Intel and Apple Silicon), and Linux
- When a tag with format `v*` is pushed, a new GitHub release is automatically created with the compiled binaries

## License

MIT 