//! Hex Standalone CLI and High-Speed Solver Binary.
//!
//! OOP Description:
//! This binary entry point provides command-line solving, performance benchmarking,
//! and GTP-style coordinate evaluation for headless Hex gameplay analysis.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use hex_engine::board::{HexBoard, BLUE, RED};
use hex_engine::search::SearchEngine;
use std::env;
use std::time::Instant;

/// Usage:
///     main()
/// Usage Example:
///     cargo run --release -- --size 11 --depth 14 --player BLUE
/// Description:
///     Parses command-line arguments and executes multi-threaded search benchmark.
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut size = 11usize;
    let mut depth = 14u8;
    let mut player = BLUE;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--size" | "-s" => {
                if i + 1 < args.len() {
                    size = args[i + 1].parse().unwrap_or(11);
                    i += 1;
                }
            }
            "--depth" | "-d" => {
                if i + 1 < args.len() {
                    depth = args[i + 1].parse().unwrap_or(14);
                    i += 1;
                }
            }
            "--player" | "-p" => {
                if i + 1 < args.len() {
                    let p_str = args[i + 1].to_uppercase();
                    player = if p_str.starts_with('R') { RED } else { BLUE };
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("==================================================");
    println!("  Hex Nash SIMD Bitboard Engine (Rust / C++)");
    println!("  Board: {}x{} | Depth: {} | Player: {}", size, size, depth, if player == RED { "RED" } else { "BLUE" });
    println!("==================================================");

    let board = HexBoard::new(size);
    let mut engine = SearchEngine::new();

    let t0 = Instant::now();
    let (best_move, score, stats) = engine.search(&board, player, depth, None);
    let elapsed = t0.elapsed().as_secs_f64();

    if let Some((r, c)) = best_move {
        let col_char = (b'A' + c as u8) as char;
        println!("Best Move: {}{} (r={}, c={})", col_char, r + 1, r, c);
    } else {
        println!("Best Move: None");
    }

    println!("Eval Score: {:.2}", score);
    println!("Nodes: {} | NPS: {} n/s | Time: {:.3}s", stats.nodes, stats.nps, elapsed);
    println!("Top Moves Leaderboard:");
    for tm in stats.top_moves.iter().take(5) {
        let col_char = (b'A' + tm.c as u8) as char;
        let note_str = tm.note.as_deref().unwrap_or("");
        println!("  #{}: {}{} | Score: {:+7.2} | {}", tm.rank, col_char, tm.r + 1, tm.score, note_str);
    }
    println!("==================================================");
}
