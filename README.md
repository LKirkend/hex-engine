# Hex Nash Engine & Solver

**Author**: Logan Kirkendall (Logan@LKAud.io)  
**License**: MIT  

A high-performance SIMD Bitboard engine, C++20 integration layer, and native hardware-accelerated SDL3 GUI for the abstract game of **Hex**, inspired by John Nash's game-theoretic proofs and combinatorial game decomposition research (*"On a decomposition method for finding winning strategy in Hex game"* by Jing Yang, Simon Liao, Mirek Pawlak).

---

## One-Command Build & Launch (`compile.sh`)

Simply run:
```bash
./compile.sh
```
*(Optional: pass a custom board dimension, e.g. `./compile.sh 11` or `./compile.sh 7`)*

This script automatically builds the Rust SIMD Bitboard core in release mode, compiles the C++ targets via CMake, and immediately launches the hardware-accelerated GUI client.

---

## Performance & Multi-Language Architecture

- **Rust SIMD Bitboard Core (`src/`)**:
  - **128-Bit Integer Register Bitboards**: Stores full $11 \times 11$ board state in 128-bit integer registers (`Bitboard128`), executing 6-neighbor dilation and flood-fill connectivity in $< 10\text{ns}$ using SIMD bit-shifts.
  - **Principal Variation Search (PVS / NegaScout)** with **Late Move Reductions (LMR)** and **Aspiration Windows**.
  - **Lock-Free Atomic Transposition Table**: Supports parallel multi-threaded Lazy SMP across all CPU cores.
  - **C-ABI Shared Library (`libhex_engine.dylib` / `.so`)**: Zero-cost dynamic linking with C++ and external frontends.
- **C++20 Native Layer & SDL3 GUI (`cpp/`)**:
  - **Hardware-Accelerated GUI (`hex_gui`)**: Built with SDL3 rendering at 60 FPS. Features diamond board layout ($A11$ on bottom, $K1$ on top), geometry-hugging thick borders, 4-sided coordinate labels, live chess-style evaluation meter, candidate move ghost previews, interactive top bar buttons, PGN move history, and clipboard import/export.
  - **Type-Safe C++20 Header (`hex::HexEngine`, `hex::HexBoard`)** and standalone native CLI (`hex_cpp_cli`).

---

## GUI Interactive Controls & Top Bar Buttons

- **`Left Arrow` / `[Undo]`**: Step backward to parent move in game tree (also `Ctrl+Z` / `Cmd+Z`).
- **`Right Arrow` / `[Redo]`**: Step forward to active child move in game tree (also `Ctrl+Shift+Z` / `Cmd+Shift+Z`).
- **`Click Move Token`**: Click on any move in the **Game Tree (PGN)** panel to navigate directly to that position.
- **`Right-Click Move Token`**: Opens a floating dropdown context menu with options:
  - **Delete** (or **Delete Branch** if following moves exist): Removes the selected node/branch from the game tree.
  - **Make Primary Branch**: Promotes the selected variation node to become the primary (mainline) branch across all parent nodes.
- **`Branching Variations`**: Playing alternative moves from any historical position automatically spawns a new variation branch (rendered in darker text).
- **`[Reset]`**: Reset board to initial empty state (or press `R`).
- **`[Clear TT]`**: Clear Transposition Table memoization cache (or press `C`).
- **`[Copy PGN]`**: Copy standard PGN algebraic notation of the active game tree path to clipboard.
- **`[Import PGN]`**: Import PGN algebraic notation directly from clipboard and replay into the tree.
- **`[Size -] / [+]`**: Adjust board size with resize confirmation dialog (3x3 to 13x13).
- **`[Depth -] / [+]`**: Adjust engine search depth dynamically without resetting search (or press `+` / `-`).
- **`[Swap Turn]`**: Swaps player starting turn order (BLUE $\leftrightarrow$ RED) and resets the board with a confirmation dialog.
- **`Tab`**: Collapse or expand the right-hand candidate leaderboard & PGN pane.
- **Left Mouse Click**: Place a stone on the hovered board cell.

---

## Automated Test Suites & Benchmarking

- **Rust Engine Suite**:
  ```bash
  cargo test --release
  ```
- **C++ Native Engine Suite**:
  ```bash
  ./build/test_cpp_engine
  ```
- **Native Benchmark Suite**:
  ```bash
  ./build/hex_benchmark
  ```
  *(Measures search throughput in nodes/second, search latency across multiple depths, and resident process RAM)*

---

## Headless CLI Solvers

- **Native C++ CLI**:
  ```bash
  ./build/hex_cpp_cli --size 11 --depth 14 --player RED
  ```
- **Rust SIMD Bitboard CLI**:
  ```bash
  cargo run --release --bin hex_cli -- --size 11 --depth 14 --player BLUE
  ```

---

## System Architecture

See [`architecture.md`](file:///Users/logankirkendall/Documents/antigravity/hex-nash-engine/architecture.md) for full module specifications, class diagrams, and method catalogs.

---

## Contributors

See [`CONTRIBUTORS.md`](file:///Users/logankirkendall/Documents/antigravity/hex-nash-engine/CONTRIBUTORS.md) for maintainers and academic citations.
