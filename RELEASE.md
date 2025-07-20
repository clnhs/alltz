# Release Process

alltz uses GitHub Actions for automated cross-platform releases.

## Pre-Release Checklist

Before creating a release, ensure:

```bash
# Pre-release checks
- ensure you're on main branch with clean working directory
- ensure CHANGELOG.md is up to date with new version
- review README.md and update if necessary
- update version in Cargo.toml if needed
- ensure all tests pass: `cargo test`
- ensure code passes formatting: `cargo fmt -- --check`  
- ensure code passes linting: `cargo clippy -- -D warnings`
- test key functionality: `cargo run -- --help` and `cargo run`
```

## Automated Release Process

### 1. Create a Release

To create a new release:

```bash
# Create and push tag (triggers GitHub Actions)
git tag v0.X.Y
git push origin v0.X.Y
```

### 2. GitHub Actions Workflow

The release workflow (`.github/workflows/release.yml`) automatically:

1. **Builds for multiple platforms:**
   - macOS Intel (x86_64-apple-darwin)
   - macOS ARM64 (aarch64-apple-darwin) 
   - Linux (x86_64-unknown-linux-gnu)
   - Windows (x86_64-pc-windows-msvc)

2. **Creates release artifacts:**
   - Platform-specific tarballs (.tar.gz for Unix, .zip for Windows)
   - SHA256 checksums for all binaries
   - GitHub release with changelog

3. **Prepares Homebrew formula:**
   - Automatically updates `homebrew-tap/Formula/alltz.rb`
   - Updates SHA256 hashes for macOS binaries
   - Creates **DRAFT** pull request for review

### 3. Verify Release

After tag is pushed, verify the automated release:

```bash
# Verify release
- check GitHub Actions completed successfully: `gh run list`
- verify release artifacts are created
- test download and installation works
```

### 4. Manual Steps

After the automated release:

1. **Review GitHub Release:** Check the auto-generated release notes and artifacts
2. **Test Homebrew Formula:** Review the draft PR and test the formula:
   ```bash
   # Test the updated formula locally
   brew install --build-from-source ./homebrew-tap/Formula/alltz.rb
   ```
3. **Update Linux SHA256:** If needed, update the Linux hash in the formula
4. **Publish to Homebrew:** When ready, mark the PR as ready for review and merge
5. **Announce:** Share the release in relevant channels

### 4. Controlled Homebrew Release

The workflow creates a **draft PR** so you have full control:
- ✅ GitHub release is published immediately
- ⏳ Homebrew formula changes are staged in a draft PR
- 🎛️ You decide when to make the formula available to users

## Homebrew Installation Options

### Option 1: Precompiled Binaries (Default)
```bash
brew tap abradburne/alltz
brew install alltz
```

### Option 2: Build from Source
For users who prefer building from source or need custom compilation:
```bash
brew tap abradburne/alltz
brew install --HEAD alltz
```

This requires Rust to be available and will compile from the latest main branch.

## Local Development Builds

For local testing during development:

```bash
# Quick local build for your architecture
./release-local.sh

# Or standard cargo build
cargo build --release
```

## Troubleshooting

### Cross-compilation Issues
If you need to build locally for multiple platforms, install targets first:

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu
```

Linux cross-compilation from macOS requires additional setup (linkers, etc.) which is why we recommend using GitHub Actions for production builds.

### GitHub Actions Failures
Common issues:
- Missing `GITHUB_TOKEN` permissions (should be automatic)
- Version conflicts (ensure tag matches version in files)
- Target platform build failures (check Rust toolchain compatibility)

### Homebrew Formula Issues
- SHA256 mismatches: The workflow calculates these automatically
- Binary naming: Ensure GitHub Actions outputs match formula expectations
- Dependencies: Only `rust` is required for source builds