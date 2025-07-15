#!/bin/bash
# alltz Release Script
# 
# ⚠️  DEPRECATED: Use GitHub Actions for production releases
# For local development builds, use ./release-local.sh instead

set -e

echo "⚠️  This script is deprecated!"
echo ""
echo "For production releases:"
echo "1. Create a git tag: git tag v0.1.0"
echo "2. Push tag: git push origin v0.1.0" 
echo "3. GitHub Actions will build for all platforms automatically"
echo ""
echo "For local development builds:"
echo "Run: ./release-local.sh"
echo ""
read -p "Continue with legacy local build? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 0
fi

VERSION="0.1.0"
NATIVE_TARGET=$(rustc -vV | sed -n 's/host: //p')
TARGETS=("$NATIVE_TARGET")

echo "🚀 Building alltz v${VERSION} for local development..."

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
    
    # Create distribution package
    cp "target/${target}/release/alltz" "dist/alltz-${target}"
    tar -czf "dist/alltz-v${VERSION}-${target}.tar.gz" -C dist "alltz-${target}"
    
    echo "✅ Created dist/alltz-v${VERSION}-${target}.tar.gz"
done

# Create universal binary for macOS (if both Intel and ARM builds exist)
if [ -f "dist/alltz-x86_64-apple-darwin" ] && [ -f "dist/alltz-aarch64-apple-darwin" ]; then
    echo "🔨 Creating universal macOS binary..."
    lipo -create -output "dist/alltz-universal" \
         "dist/alltz-x86_64-apple-darwin" \
         "dist/alltz-aarch64-apple-darwin"
    tar -czf "dist/alltz-v${VERSION}-universal-macos.tar.gz" -C dist "alltz-universal"
    echo "✅ Created universal macOS binary"
fi

# Generate checksums
cd dist
shasum -a 256 *.tar.gz > checksums.txt
cd ..

echo ""
echo "🎉 Release packages created in dist/:"
ls -la dist/*.tar.gz
echo ""
echo "📋 Checksums:"
cat dist/checksums.txt