# lowell - UKI/initramfs tooling

OCI-native tools for building hermetic, host-agnostic boot artifacts (UKIs and initramfs). This is a Rust project with CLI and library components.

Always follow these instructions first and fallback to search or bash commands only when you encounter information that does not match what is documented here.

## Working Effectively

### Prerequisites and Setup
- Install Rust stable toolchain: `rustup default stable`
- Install just task runner: `cargo install just`
- Set PATH: `export PATH="$HOME/.cargo/bin:$PATH"`

### Build, Test, and Lint Commands
Execute these commands in order for a complete development workflow:

```bash
# Clean build from scratch (NEVER CANCEL - takes 30 seconds)
just build
# OR: cargo build -p lowell-cli --release --locked

# Format code (required before commit)  
just fmt
# OR: cargo fmt --all

# Lint with clippy (required before commit)
just clippy  
# OR: cargo clippy --all-targets --all-features -- -D warnings

# Run tests (required before commit)
just test
# OR: cargo test --all

# Run complete CI pipeline (fmt + clippy + test)
just ci
```

### Build Times (CRITICAL - Set Appropriate Timeouts)
- **Clean build**: 25-30 seconds - NEVER CANCEL, set timeout to 60+ minutes
- **Incremental build**: <1 second
- **Clean CI pipeline**: 22 seconds - NEVER CANCEL, set timeout to 45+ minutes  
- **Incremental CI**: <1 second
- **Format check**: <1 second
- **Clippy (clean)**: 10-13 seconds
- **Tests (clean)**: 11 seconds

### Running the CLI
The main binary is `lowell` - a CLI tool for inspecting UKI files:

```bash
# Build first
just build

# Basic usage
./target/release/lowell inspect uki --file /path/to/vmlinuz.efi

# Available options
--format json|human|json-pretty    # Output format
--verbose                          # Show more fields in human output  
--log-level debug|info|warn|error  # Global log level

# Examples
./target/release/lowell --version
./target/release/lowell --help
./target/release/lowell --log-level debug inspect uki --help
```

## Validation

### Manual Testing Requirements
- **ALWAYS** build and test any changes before committing
- **ALWAYS** run the complete CI pipeline: `just ci`
- **Test CLI functionality**: Run `./target/release/lowell --version` and `--help` to verify basic operation
- **Error handling**: Test with non-existent files to verify proper error messages
- **All format options**: Test `--format human`, `--format json`, and `--format json-pretty`

### Integration Tests
The project includes integration tests that require real UKI files:
- Unit tests always run: `cargo test`
- Integration test (ignored by default): requires `UKI_PATH` environment variable
- To run integration test: `UKI_PATH=/path/to/real.efi cargo test -- --ignored`
- Smoke test helper: `just smoke /path/to/uki.efi`

## Repository Structure

### Key Directories
- `lowell-cli/`: CLI application crate
- `lowell-core/`: Library crate with core functionality  
- `lowell-core/src/formats/`: File format parsing (PE, initramfs, os-release)
- `lowell-core/src/inspect/`: UKI inspection logic
- `.github/workflows/`: CI/CD pipelines
- `profiles/`: Configuration profiles (kvm-ostree.toml)

### Important Files
- `Justfile`: Task runner recipes (preferred over direct cargo commands)
- `Cargo.toml`: Workspace configuration
- `lowell-cli/Cargo.toml`: CLI dependencies
- `lowell-core/Cargo.toml`: Library dependencies
- `.github/workflows/ci.yml`: CI pipeline (runs on Ubuntu and macOS)
- `.github/workflows/release.yml`: Release builds for multiple platforms

### Common File Locations
When making changes, these files are frequently modified together:
- CLI command definitions: `lowell-cli/src/cli/`
- Core inspection logic: `lowell-core/src/inspect/uki.rs`
- File format parsers: `lowell-core/src/formats/`

## CI/CD Pipeline

The GitHub Actions CI runs:
1. `just ci` (which includes fmt, clippy, test)
2. Runs on Ubuntu and macOS
3. Uses Rust stable toolchain
4. Leverages rust-cache for faster builds

**CI Requirements**: 
- All code must pass `cargo fmt --all --check`
- All code must pass `cargo clippy --all-targets --all-features -- -D warnings` 
- All tests must pass `cargo test --all`

## Development Workflow

### Pre-commit Checklist
Always run these commands before committing:
```bash
just ci  # Runs fmt, clippy, and test
```

### Code Style Guidelines
- Use `tracing` for logs, prefer structured logs over `println!`
- Favor `anyhow`/`thiserror` for error handling; avoid `unwrap()` in library code
- Keep commits focused; Conventional Commits appreciated but not required
- Add DCO `Signed-off-by` line to commit messages

### Making Changes
1. **Always build first**: `just build`
2. **Make your changes**
3. **Test immediately**: `just ci`
4. **Test CLI manually**: Run `./target/release/lowell --help` and test relevant functionality
5. **Add/update tests** when changing behavior
6. **Update documentation** if needed

## Common Tasks Quick Reference

```bash
# List all available tasks
just --list

# Build only
just build

# Quick JSON inspect (for timing/performance testing)
just fast /path/to/uki.efi

# Inspect with custom log level
just inspect /path/to/uki.efi
RUST_LOG=debug just inspect /path/to/uki.efi

# Environment-based UKI path
echo 'UKI_PATH=/path/to/uki.efi' > .env  
just inspect-env

# Cross-platform release builds (like CI)
cargo build -p lowell-cli --release --locked --target x86_64-unknown-linux-gnu
```

## Project Status

Currently implements:
- **CLI inspection**: `lowell inspect uki --file <path>`
- **Output formats**: human, json, json-pretty  
- **Inspection data**: architecture, PE32+ format, signatures, cmdline, os-release, section offsets/sizes/hashes
- **Compression detection**: gzip, xz, zstd, uncompressed initramfs

Planned features:
- `lowell inject uki`: modify initramfs and rebuild UKI
- `lowell build`: hermetic initramfs + UKI building

## Troubleshooting

### Build Issues
- **"just: command not found"**: Install with `cargo install just`
- **Network issues**: All dependencies should build offline after initial `cargo build`
- **Permission denied**: Ensure `target/release/lowell` is executable

### Test Issues
- **Integration test skipped**: This is normal - requires real UKI files via `UKI_PATH`
- **Clippy warnings**: Treated as errors, must be fixed before commit

### Performance
- First builds take 25-30 seconds
- Subsequent builds are very fast (<1 second)
- Clean the build cache with `cargo clean` if issues persist