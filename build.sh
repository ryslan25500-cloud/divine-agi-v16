#!/bin/bash
# Divine AGI V16 — Local Build Script
# Run this on your machine to compile the binary

set -e

echo "🧬 Divine AGI V16 — Local Build"
echo "================================"

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Install nightly if needed
echo "📦 Setting up Rust nightly..."
rustup install nightly 2>/dev/null || true
rustup default nightly

# Show version
echo "🔧 Rust version:"
rustc --version
cargo --version

# Build release
echo ""
echo "🔨 Building release binary..."
cargo build --release

# Check result
if [ -f "target/release/divine-agi" ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    
    # Copy to project root for Docker
    cp target/release/divine-agi ./divine-agi
    
    # Show binary info
    ls -lh divine-agi
    file divine-agi
    
    echo ""
    echo "🚀 Ready to deploy!"
    echo "   Run: git add . && git commit -m 'V16 binary' && git push"
else
    echo "❌ Build failed!"
    exit 1
fi
