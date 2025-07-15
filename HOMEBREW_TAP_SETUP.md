# 🍺 Homebrew Tap Setup Guide for alltz

This guide walks you through creating a complete Homebrew tap for alltz.

## Prerequisites

- GitHub account
- GitHub CLI (`brew install gh`)
- Rust toolchain for cross-compilation

## Step-by-Step Setup

### 1. Create Main Repository (if not done)
```bash
# In your alltz project
git init
git add .
git commit -m "Initial commit"
git remote add origin https://github.com/abradburne/alltz.git
git push -u origin main
```

### 2. Create Homebrew Tap Repository
```bash
# Create new repository on GitHub named: homebrew-alltz
# Clone it locally
git clone https://github.com/abradburne/homebrew-alltz.git
cd homebrew-alltz

# Copy the tap files we created
cp /path/to/alltz/homebrew-tap/* .
```

### 3. Build and Release
```bash
# In your alltz project directory
./create-github-release.sh
```

This will:
- Build binaries for macOS (Intel + ARM) and Linux
- Generate SHA256 checksums
- Create GitHub release with assets
- Provide SHA256 hashes for the formula

### 4. Update Homebrew Formula
```bash
# Copy the SHA256 hashes from step 3 output
# Edit homebrew-alltz/Formula/alltz.rb and replace:
# - SHA256_ARM64_HASH with ARM64 macOS hash
# - SHA256_X86_64_HASH with Intel macOS hash
# - SHA256_LINUX_HASH with Linux hash
# - abradburne with your actual GitHub username

git add .
git commit -m "Add alltz formula v0.1.0"
git push origin main
```

### 5. Test Installation
```bash
# Test the tap locally
brew tap abradburne/alltz
brew install alltz

# Verify installation
alltz --version
alltz list
```

## Formula Template

Here's the template for `Formula/alltz.rb`:

```ruby
class Alltz < Formula
  desc "🌍 Terminal-based timezone viewer for developers and remote teams"
  homepage "https://github.com/abradburne/alltz"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/abradburne/alltz/releases/download/v0.1.0/alltz-v0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA"
    else
      url "https://github.com/abradburne/alltz/releases/download/v0.1.0/alltz-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_INTEL_SHA"
    end
  end

  on_linux do
    url "https://github.com/abradburne/alltz/releases/download/v0.1.0/alltz-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_WITH_LINUX_SHA"
  end

  def install
    bin.install "alltz-#{Hardware::CPU.arch}-apple-darwin" => "alltz" if OS.mac?
    bin.install "alltz-x86_64-unknown-linux-gnu" => "alltz" if OS.linux?
  end

  test do
    assert_match "alltz 0.1.0", shell_output("#{bin}/alltz --version")
    assert_match "Available Timezones", shell_output("#{bin}/alltz list")
  end
end
```

## User Installation

Once your tap is set up, users can install alltz with:

```bash
# Add your tap
brew tap abradburne/alltz

# Install alltz
brew install alltz

# Or install directly
brew install abradburne/alltz/alltz
```

## Updating for New Releases

For each new version:

1. Update version in `Cargo.toml`
2. Run `./create-github-release.sh`
3. Update `Formula/alltz.rb` with new version and SHA256 hashes
4. Commit and push the tap repository

## Tips

- **Repository naming**: Homebrew taps must be named `homebrew-<name>`
- **Testing**: Always test your formula before publishing
- **Documentation**: Keep your tap README updated
- **Automation**: Consider GitHub Actions for automatic formula updates

## Troubleshooting

- **SHA256 mismatch**: Regenerate checksums after rebuilding
- **Binary not found**: Ensure binary names match in tar archives
- **Permission denied**: Check executable permissions on binaries

## Advanced: GitHub Actions Automation

Consider setting up GitHub Actions to automatically update your tap when you create new releases in the main repository.