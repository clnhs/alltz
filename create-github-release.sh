#!/bin/bash
# GitHub Release Creation Script for alltz

set -e

VERSION="0.1.0"
REPO="your-username/alltz"
TAG="v${VERSION}"

echo "🚀 Creating GitHub release for alltz v${VERSION}..."

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is required. Install with: brew install gh"
    exit 1
fi

# Check if user is logged in
if ! gh auth status &> /dev/null; then
    echo "❌ Please login to GitHub CLI: gh auth login"
    exit 1
fi

# Build release assets
echo "📦 Building release assets..."
./release-with-sha.sh

# Create GitHub release
echo "🏷️ Creating GitHub release..."
gh release create "$TAG" \
    --title "alltz v${VERSION}" \
    --notes "## 🌍 alltz v${VERSION}

### Features
- 🌤️ **Weather Integration** - Real-time weather icons for each timezone
- 🎨 **6 Color Themes** - Default, Ocean, Forest, Sunset, Cyberpunk, Monochrome
- 🕐 **DST Indicators** - Visual spring forward (⇈) and fall back (⇊) arrows
- 📍 **Local Time Display** - Shows scrubbed time in your timezone with day and UTC offset
- 💻 **CLI Commands** - \`list\`, \`time <city>\`, \`zone <city>\`
- 💾 **Persistent Config** - Saves your timezone list and preferences

### Installation

#### Homebrew (macOS/Linux)
\`\`\`bash
brew tap your-username/alltz
brew install alltz
\`\`\`

#### Manual Installation
Download the appropriate binary for your platform below and extract to your PATH.

#### From Source
\`\`\`bash
cargo install --git https://github.com/${REPO}
\`\`\`

### Usage
\`\`\`bash
# Launch interactive TUI
alltz

# Show available timezones
alltz list

# Check time in specific city
alltz time London
\`\`\`

### TUI Controls
- \`?\` - Show help
- \`h/l\` or \`←/→\` - Scrub timeline
- \`j/k\` or \`↑/↓\` - Navigate timezones
- \`c\` - Cycle color themes
- \`w\` - Toggle weather
- \`q\` - Quit

Full documentation available in [INSTALL.md](https://github.com/${REPO}/blob/main/INSTALL.md)" \
    dist/*.tar.gz

echo "✅ GitHub release created successfully!"
echo "🔗 View at: https://github.com/${REPO}/releases/tag/${TAG}"

# Extract SHA256 hashes for Homebrew formula update
echo ""
echo "📋 SHA256 hashes for Homebrew formula:"
echo "======================================"
cd dist
while IFS= read -r line; do
    sha=$(echo "$line" | cut -d' ' -f1)
    file=$(echo "$line" | cut -d' ' -f2)
    
    case "$file" in
        *aarch64-apple-darwin*)
            echo "ARM64 macOS: $sha"
            ;;
        *x86_64-apple-darwin*)
            echo "Intel macOS: $sha"
            ;;
        *x86_64-unknown-linux-gnu*)
            echo "Linux: $sha"
            ;;
    esac
done < checksums.txt
cd ..

echo ""
echo "📝 Next steps:"
echo "1. Update homebrew-tap/Formula/alltz.rb with the SHA256 hashes above"
echo "2. Commit and push the tap repository"
echo "3. Users can now install with: brew tap your-username/alltz && brew install alltz"