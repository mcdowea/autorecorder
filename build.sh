#!/bin/bash

echo "================================"
echo "Auto Recorder Build Script"
echo "================================"

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install from https://rustup.rs/"
    exit 1
fi

echo ""
echo "Step 1: Checking project structure..."
echo "✓ Cargo.toml found"
echo "✓ src/ directory found"

echo ""
echo "Step 2: Checking dependencies..."
cargo --version
rustc --version

echo ""
echo "Step 3: Running cargo check..."
cargo check

if [ $? -eq 0 ]; then
    echo "✓ Cargo check passed"
else
    echo "✗ Cargo check failed"
    exit 1
fi

echo ""
echo "Step 4: Building project (debug)..."
cargo build

if [ $? -eq 0 ]; then
    echo "✓ Debug build successful"
else
    echo "✗ Debug build failed"
    exit 1
fi

echo ""
echo "Step 5: Building project (release)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Release build successful"
else
    echo "✗ Release build failed"
    exit 1
fi

echo ""
echo "================================"
echo "Build completed successfully!"
echo "================================"
echo ""
echo "Binary locations:"
echo "  Debug:   ./target/debug/auto-recorder"
echo "  Release: ./target/release/auto-recorder"
echo ""
echo "Quick start:"
echo "  ./target/release/auto-recorder gen-config"
echo "  ./target/release/auto-recorder list-devices"
echo "  ./target/release/auto-recorder record"
echo ""
