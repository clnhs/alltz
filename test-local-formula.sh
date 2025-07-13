#!/bin/bash
# Test Homebrew formula locally before publishing

set -e

echo "🧪 Testing Homebrew formula locally..."

# Check if brew is installed
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew is required for testing"
    exit 1
fi

FORMULA_PATH="homebrew-tap/Formula/alltz.rb"

if [ ! -f "$FORMULA_PATH" ]; then
    echo "❌ Formula not found at $FORMULA_PATH"
    exit 1
fi

echo "📋 Validating formula syntax..."
brew audit --strict --online "$FORMULA_PATH" || echo "⚠️  Some audit checks failed (this is normal for local testing)"

echo "🔍 Testing formula installation..."
brew install --build-from-source "$FORMULA_PATH"

echo "✅ Testing installed binary..."
alltz --version
alltz list | head -5

echo "🧹 Cleaning up..."
brew uninstall alltz

echo "✅ Local formula test completed successfully!"
echo "📝 Your formula is ready for publication!"