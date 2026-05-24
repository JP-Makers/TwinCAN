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

# Step 2: Package the Python UI with Nuitka
echo "============================================"
echo " Packaging Python UI with Nuitka..."
echo "============================================"
python3 -m nuitka --standalone --onefile \
    --enable-plugin=tk-inter \
    --include-data-file="target/release/backend=target/release/backend" \
    --include-data-dir="assets=assets" \
    --output-filename="TwinCAN_v${version}" \
    --output-dir="$output_dir" \
    --remove-output \
    ui/ui.py

if [ $? -ne 0 ]; then
    echo "Error: Nuitka packaging failed."
    exit 1
fi

echo "============================================"
echo " Build complete!"
echo " Output: dist/TwinCAN_v${version}"
echo "============================================"