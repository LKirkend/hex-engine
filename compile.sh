#!/usr/bin/env bash
set -e

# ==============================================================================
# Hex Nash Engine Compilation and Launch Script
# Author: Logan Kirkendall <Logan@LKAud.io>
# Builds the Rust SIMD Bitboard core, compiles C++ targets via CMake,
# and immediately launches the hardware-accelerated C++ GUI client.
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== [1/2] Building Rust SIMD Bitboard Core ==="
cargo build --release

echo "=== [2/2] Configuring and Compiling C++ GUI via CMake ==="
cmake -B build cpp
cmake --build build

echo "=== Launching Hex Nash C++ Native GUI ==="
./build/hex_gui "$@"
