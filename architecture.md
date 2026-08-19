# System Architecture - Hex Nash Engine (Pure Rust & C++20)

## Overview
The **Hex Nash Engine** is a high-performance Hex game engine, solver, and native hardware-accelerated GUI built purely in **Rust** (SIMD Bitboards) and **C++20** (SDL3 Graphical Interface).

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                cpp/src/gui_main.cpp                                    │
│                    (Launches Native C++20 Hardware-Accelerated GUI)                    │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                   ┌────────────────────────┴────────────────────────┐
                   ▼                                                 ▼
        ┌──────────────────────┐                          ┌──────────────────────┐
        │ cpp/include/hex_gui  │                          │   cpp/src/main.cpp   │
        │ (SDL3 Window & Async │                          │ (Standalone Native   │
        │  Search Worker Loop) │                          │  C++ CLI Solver Bin) │
        └──────────┬───────────┘                          └──────────┬───────────┘
                   │                                                 │
     ┌─────────────┼────────────────────────┬────────────────────────┤
     ▼             ▼                        ▼                        │
┌──────────┐ ┌──────────┐ ┌──────────────────────┐┌──────────────────┴───┐
│gui_render│ │gui_modal │ │cpp/include/gui_panel ││cpp/include/gui_top_bar│
│(Diamond  │ │(Confirm  │ │(Collapsible Move     ││(Action Buttons, Size/│
│ Hex Board│ │ Modal)   │ │ Leaderboard & PGN)   ││ Depth Spinners, Turn)│
└────┬─────┘ └────┬─────┘ └──────────┬───────────┘└──────────┬───────────┘
     │            │                  │                       │
     └────────────┴──────────────────┼───────────────────────┘
                                     ▼
                         ┌──────────────────────┐
                         │cpp/include/hex_engine│
                         │(C++20 RAII Wrapper)  │
                         └───────────┬──────────┘
                                     │
                                     ▼
                         ┌──────────────────────┐
                         │      src/lib.rs      │
                         │ (C-ABI Dynamic Link) │
                         └───────────┬──────────┘
                                     │
                  ┌──────────────────┼──────────────────┬──────────────────┐
                  ▼                  ▼                  ▼                  ▼
         ┌──────────────────┐┌──────────────────┐┌──────────────────┐┌──────────────────┐
         │ src/bitboard.rs  ││   src/board.rs   ││ src/evaluator.rs ││  src/search.rs   │
         │(128-Bit SIMD Dila││(Bitboards, Zob-  ││(Resistance+Heur- ││(NMP, Futility,   │
         │ tion & Neighbor) ││ rist, Fast Wins) ││ istic Dual Eval) ││ LMP, Hist/Killer)│
         └──────────────────┘└──────────────────┘└────────┬─────────┘└──────────────────┘
                                     │                    │                  │
                                     ▼                    ▼                  ▼
                             ┌──────────────────┐┌──────────────────┐┌──────────────────┐
                             │    src/tt.rs     ││src/resistance.rs ││  src/openings.rs │
                             │(Lock-Free Atomic ││(Gauss-Seidel     ││(Master 11x11 Tree│
                             │ Transposition Tbl││ Network Solver)  ││ & Solved Openings│
                             └──────────────────┘└──────────────────┘└──────────────────┘
```

---

## File & Module Index

### 1. `src/bitboard.rs`
**Description**: 128-bit SIMD Bitboard operations (`Bitboard128`). Executes 6-direction hexagonal neighbor expansions using bit-shifts and row/col bitmasks in $< 10\text{ns}$.

### 2. `src/board.rs`
**Description**: `HexBoard` struct maintaining Red and Blue SIMD bitboards, turn tracking, 64-bit Zobrist hashes, and sub-20ns flood-fill terminal win detection. Exposes `zobrist_key(idx, player)` and `zobrist_player_key(player)` for incremental hash computation without board cloning.

### 3. `src/evaluator.rs`
**Description**: `HexEvaluator` dual-mode evaluation: `evaluate_for_player()` blends 60% resistance-based network evaluation with 40% heuristic features (shortest-path, center dominance, bridge templates). `evaluate_fast()` uses heuristic-only mode for deep leaf nodes. Features 0-1 BFS shortest path seeded with precomputed Edge-2 and Edge-3 templates at source/sink boundaries, center core dominance, 2-bridge virtual connection templates, decisive zero-distance completion detection, and sudden-death detection.

### 3a. `src/resistance.rs`
**Description**: `ResistanceEvaluator` models the Hex board as an electrical resistance network. Own stones are superconductors (0Ω), empty cells are unit resistors (1Ω), opponent stones are insulators (∞Ω). Uses 16-iteration Gauss-Seidel relaxation to solve Kirchhoff equations. Models direct stone-to-stone 2-bridge virtual links ($50.0\,\Omega^{-1}$ conductance) and precomputed edge-template rail coupling ($80.0\,\Omega^{-1}$ conductance) between border stones and rail conductors. Provides highly accurate continuous evaluation that naturally captures path multiplicity and dead cells.

### 3b. `src/patterns.rs`
**Description**: `HexPatternMatcher` tactical pattern recognition including precomputed Edge-2, Edge-3, Edge-4, and Edge-5 lookup tables (`RED_NORTH_TEMPLATES`, `RED_SOUTH_TEMPLATES`, `BLUE_WEST_TEMPLATES`, `BLUE_EAST_TEMPLATES`), direct source/sink connectivity checks (`is_stone_connected_to_source_edge`, `is_stone_connected_to_sink_edge`), genuine opponent carrier disruption verification (`count_opponent_carrier_disruptions`), symmetric compulsory 2-bridge and edge-template carrier defense (`get_compulsory_carrier_response`), futile carrier attack penalties ($-65.0$), genuine 2-bridge cuts ($+110.0$), border cutoff wall, corner sealing, trailing ladder, and strategic guidance.

### 4. `src/tt.rs`
**Description**: `TranspositionTable` lock-free atomic memoization cache supporting parallel multi-threaded Lazy SMP search across CPU cores.

### 5. `src/openings.rs`
**Description**: `HexOpeningBook` containing game-theoretic solved opening moves (3x3 to 10x10) and the 11x11 master opening book tree.

### 6. `src/search.rs`
**Description**: `SearchEngine` executing parallel Lazy SMP iterative deepening minimax with PVS, **Null Move Pruning** (R=2/3, guarded by tactical danger), **Futility Pruning** (depth 1-2, margins 55/110, guarded by tactical danger), **Late Move Pruning** (depth ≤5, limit 6+depth×3, guarded by tactical danger), **History Heuristic** (depth²-weighted cutoff tracking), **Killer Move Heuristic** (2 killers/ply), and **LMR** (reduction 1-2 for late moves). Aspiration window widened to 35.0 to prevent re-search thrashing. Tighter branching: 32→24→18→14 candidates by depth. `get_initial_candidates` extracts and ranks candidate moves instantly via TT memoization lookup and fast heuristic ordering for zero-latency UI updates upon move navigation.

### 7. `src/lib.rs` & `src/main.rs`
**Description**: `src/lib.rs` exports C-ABI dynamic symbols (`hex_engine_*`), including `hex_engine_get_initial_candidates` and streaming callbacks; `src/main.rs` is the standalone Rust CLI solver binary (`hex_cli`).

### 8. `cpp/include/hex_engine.hpp`, `cpp/src/main.cpp`, `cpp/src/benchmark.cpp`
**Description**: Modern C++20 header wrapper (`hex::HexEngine`, `hex::HexBoard`), CMake configuration, standalone C++ solver binary (`hex_cpp_cli`), and standalone benchmarking suite (`hex_benchmark`) measuring throughput (NPS), latency, and resident memory (RSS).

### 9. `cpp/include/hex_gui.hpp`, `gui_move_tree.hpp`, `gui_context_menu.hpp`, `gui_renderer.hpp`, `gui_panel.hpp`, `gui_modal.hpp`, `gui_top_bar.hpp`, `cpp/src/gui_main.cpp`
**Description**: Native hardware-accelerated SDL3 GUI featuring flat-topped diamond hexagon canvas, geometry-hugging borders, 4-sided coordinate labels, chess-style evaluation meter with amplified non-linear scaling ($\tanh$, reaching 2/3rds bar height at $\pm 3.0$ eval) and in-bar evaluation number riding the division line snapped to the winning color's edge, candidate move ghost previews, opening book indicators with negative book cutouts, top bar toolbar with Board Size & Search Depth incrementing number fields, "Swap Turn" toolbar button with confirmation modal to swap starting colors and reset, modal resize confirmation dialog, interactive chess-viewer style MoveTree (`MoveTree`, `MoveNode`, `MoveTokenLayout`), floating `ContextMenu` dropdown popup on right-click ("Delete" / "Delete Branch" vs "Make Primary Branch"), Left/Right arrow key cycling between moves, clickable move tokens with darker text for branching variations, zero-latency instant position cache and candidate move switching on clicks/undo/redo, and clipboard PGN import/export with starting color preservation (`[First "Red"]` / `[First "Blue"]`).

### 10. `benchmarks/`
**Description**: Performance benchmarking directory containing uniquely dated regression and throughput audit reports (`benchmark-*.md`), documenting NPS trends, latency, memory usage, and optimization milestones.

