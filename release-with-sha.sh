#!/bin/bash
# alltz Release Script with SHA256 generation for Homebrew

set -e

VERSION="0.1.2"
TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-unknown-linux-gnu")

echo "🚀 Building alltz v${VERSION} for multiple targets..."

# Clean previous builds
cargo clean
rm -rf dist
mkdir -p dist

# Build for each target
for target in "${TARGETS[@]}"; do
    echo "📦 Building for ${target}..."
    
    # Install target if not present
    rustup target add "$target" 2>/dev/null || true
    
    # Build release binary
    cargo build --release --target "$target"
    
    # Create properly named binary for Homebrew
    cp "target/${target}/release/alltz" "dist/alltz-${target}"
    
    # Create distribution package
    tar -czf "dist/alltz-v${VERSION}-${target}.tar.gz" -C dist "alltz-${target}"
    
    echo "✅ Created dist/alltz-v${VERSION}-${target}.tar.gz"
done

# Generate checksums
cd dist
echo "📋 Generating SHA256 checksums..."
shasum -a 256 *.tar.gz > checksums.txt

echo ""
echo "🎉 Release packages created:"
ls -la *.tar.gz

echo ""
echo "📋 SHA256 Checksums for Homebrew formula:"
echo "========================================="
while IFS= read -r line; do
    sha=$(echo "$line" | cut -d' ' -f1)
    file=$(echo "$line" | cut -d' ' -f2)
    
    case "$file" in
        *aarch64-apple-darwin*)
            echo "ARM64 macOS SHA256: $sha"
            ;;
        *x86_64-apple-darwin*)
            echo "Intel macOS SHA256: $sha"
            ;;
        *x86_64-unknown-linux-gnu*)
            echo "Linux SHA256: $sha"
            ;;
    esac
done < checksums.txt

echo ""
echo "📝 Copy these SHA256 values to your Homebrew formula!"

cd ..