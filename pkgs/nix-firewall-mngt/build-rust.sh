#!/usr/bin/env sh
set -ex

SOURCE_DIR="$1"
BUILD_DIR="$2"
OUTPUT="$3"

echo "SOURCE_DIR: $SOURCE_DIR"
echo "BUILD_DIR: $BUILD_DIR"
echo "OUTPUT: $OUTPUT"
echo "PWD: $(pwd)"

cd "$SOURCE_DIR"
cargo build --release --target-dir "$BUILD_DIR/target"

echo "Build completed"

# Copy to the absolute path (OUTPUT must be an absolute or relative path to BUILD_DIR)
if [ -f "$OUTPUT" ]; then
    echo "OUTPUT already exists at: $OUTPUT"
else
    echo "Creating OUTPUT at: $BUILD_DIR/$OUTPUT"
    cp -v "$BUILD_DIR/target/release/nix-firewall-mngt" "$BUILD_DIR/$OUTPUT"
fi

echo "Verifying copy:"
ls -la "$BUILD_DIR/$OUTPUT" || ls -la "$OUTPUT" || echo "File not found!"
