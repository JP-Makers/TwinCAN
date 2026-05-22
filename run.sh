#!/bin/bash

# Navigate to script directory
cd "$(dirname "$0")"

# Get the version number from Cargo.toml
version=$(grep '^version =' Cargo.toml | cut -d '"' -f 2)

# Define output directory
output_dir="dist"
mkdir -p "$output_dir"

# Step 1: Compile the Rust backend in release mode
echo "============================================"
echo " Building Rust backend in release mode..."
echo "============================================"
cargo build --release

if [ $? -ne 0 ]; then
    echo "Error: Rust backend build failed."
    exit 1
fi
echo "Rust backend built successfully."

# Step 2: Package the Python UI with PyInstaller
echo "============================================"
echo " Packaging Python UI with PyInstaller..."
echo "============================================"
python3 -m PyInstaller --noconsole --onefile \
    --add-data "target/release/backend:target/release" \
    -n "TwinCAN_v${version}" \
    ui/ui.py

if [ $? -ne 0 ]; then
    echo "Error: PyInstaller packaging failed."
    exit 1
fi

echo "============================================"
echo " Build complete!"
echo " Output: dist/TwinCAN_v${version}"
echo "============================================"