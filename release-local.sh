#!/bin/bash
# Local development release script
# For production releases, use GitHub Actions instead

set -e

VERSION="0.1.0"
NATIVE_TARGET=$(rustc -vV | sed -n 's/host: //p')

echo "🚀 Building alltz v${VERSION} for local development..."
echo "ℹ️  Target: $NATIVE_TARGET"

# Clean and build
cargo clean
cargo build --release

# Create local distribution
mkdir -p dist
cp target/release/alltz "dist/alltz-${NATIVE_TARGET}"
tar -czf "dist/alltz-v${VERSION}-${NATIVE_TARGET}.tar.gz" -C dist "alltz-${NATIVE_TARGET}"

echo "✅ Local build completed: dist/alltz-v${VERSION}-${NATIVE_TARGET}.tar.gz"
echo ""
echo "📝 For production releases:"
echo "1. Create a git tag: git tag v${VERSION}"
echo "2. Push tag: git push origin v${VERSION}"
echo "3. GitHub Actions will automatically build for all platforms"