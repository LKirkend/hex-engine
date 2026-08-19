//! ============================================================================
//! Automated Hex Head-to-Head Tournament Match Arena
//!
//! Description:
//!     Automated CLI tool to pit the Hex Nash Engine against the Nintendo Impossible
//!     Computer (NIC) Replica in multi-game tournaments across diverse openings.
//!
//! Features:
//!     - Alternating pairwise color matches (Nash as Blue vs NIC as Red, and vice versa).
//!     - Win-rate statistics, average game plies, move latencies, and node counts.
//!     - Generates PGN game records for deep tactical divergence analysis.
//!
//! Usage:
//!     cargo run --release --bin arena -- [OPTIONS]
//!     cargo run --release --bin arena -- --games 20 --depth 6 --size 11
//!
//! Author: Logan Kirkendall (Logan@LKAud.io)
//! License: MIT
//! ============================================================================

use std::env;
use std::time::Instant;

use hex_engine::board::{HexBoard, BLUE, RED};
use hex_engine::nic_replica::NicReplicaEngine;
use hex_engine::search::SearchEngine;

/// Formats row and column indices into Hex PGN notation (e.g. (5, 5) -> "F6").
fn coord_to_pgn(r: usize, c: usize) -> String {
    let col_char = (b'a' + c as u8) as char;
    format!("{}{}", col_char, r + 1)
}

/// Represents the result of an arena match.
#[allow(dead_code)]
struct MatchResult {
    match_num: usize,
    nash_player: u8,
    nic_player: u8,
    winner: u8,
    opening_move: String,
    total_plies: usize,
    duration_ms: u128,
    pgn: String,
}

fn run_single_game(
    match_num: usize,
    size: usize,
    nash_player: u8,
    nash_depth: usize,
    nic_depth: usize,
    opening: Option<(usize, usize)>,
    verbose: bool,
) -> MatchResult {
    let mut board = HexBoard::new(size);
    let mut nash_engine = SearchEngine::new();
    let mut nic_engine = NicReplicaEngine::with_depth(nic_depth);

    let nic_player = if nash_player == BLUE { RED } else { BLUE };
    let mut current_player = BLUE;
    let mut move_history: Vec<(usize, usize)> = Vec::new();
    let start_time = Instant::now();

    // Place opening move if specified
    let mut opening_str = String::from("Standard");
    if let Some((or, oc)) = opening {
        board.place_move(or, oc, current_player);
        move_history.push((or, oc));
        opening_str = coord_to_pgn(or, oc);
        current_player = if current_player == BLUE { RED } else { BLUE };
    }

    let mut ply = move_history.len();
    while !board.is_game_over() && ply < size * size {
        ply += 1;
        let best_move = if current_player == nash_player {
            let (bm, _, _) = nash_engine.search(&board, nash_player, nash_depth as u8, None);
            bm
        } else {
            let (bm, _) = nic_engine.select_move(&board, nic_player);
            bm
        };

        match best_move {
            Some((r, c)) => {
                if !board.place_move(r, c, current_player) {
                    eprintln!("Error: Illegal move ({}, {}) attempted by player {}", r, c, current_player);
                    break;
                }
                move_history.push((r, c));
                if verbose {
                    let agent_name = if current_player == nash_player { "Nash" } else { "NIC" };
                    let color_name = if current_player == BLUE { "Blue" } else { "Red" };
                    println!(
                        "   Ply {:2}: {} ({}) plays {}",
                        ply,
                        agent_name,
                        color_name,
                        coord_to_pgn(r, c)
                    );
                }
            }
            None => {
                eprintln!("Error: No legal move found for player {}", current_player);
                break;
            }
        }

        current_player = if current_player == BLUE { RED } else { BLUE };
    }

    let winner = board.get_winner();
    let duration_ms = start_time.elapsed().as_millis();

    // Construct PGN string
    let mut pgn = format!(
        "[Game \"Hex\"]\n[Size \"{}x{}\"]\n[First \"Blue\"]\n[Blue \"{}\"]\n[Red \"{}\"]\n\n",
        size,
        size,
        if nash_player == BLUE { "NashEngine" } else { "NicReplica" },
        if nash_player == RED { "NashEngine" } else { "NicReplica" }
    );

    for (i, &(r, c)) in move_history.iter().enumerate() {
        if i % 2 == 0 {
            pgn.push_str(&format!("{}. {} ", (i / 2) + 1, coord_to_pgn(r, c)));
        } else {
            pgn.push_str(&format!("{} ", coord_to_pgn(r, c)));
        }
    }

    MatchResult {
        match_num,
        nash_player,
        nic_player,
        winner,
        opening_move: opening_str,
        total_plies: move_history.len(),
        duration_ms,
        pgn,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut num_games = 20usize;
    let mut search_depth = 6usize;
    let mut nic_depth = 3usize;
    let mut board_size = 11usize;
    let mut verbose = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--games" | "-g" => {
                if i + 1 < args.len() {
                    num_games = args[i + 1].parse().unwrap_or(20);
                    i += 1;
                }
            }
            "--depth" | "-d" => {
                if i + 1 < args.len() {
                    search_depth = args[i + 1].parse().unwrap_or(6);
                    i += 1;
                }
            }
            "--nic-depth" => {
                if i + 1 < args.len() {
                    nic_depth = args[i + 1].parse().unwrap_or(3);
                    i += 1;
                }
            }
            "--size" | "-s" => {
                if i + 1 < args.len() {
                    board_size = args[i + 1].parse().unwrap_or(11);
                    i += 1;
                }
            }
            "--verbose" | "-v" => {
                verbose = true;
            }
            "--help" | "-h" => {
                println!("Usage: cargo run --release --bin arena -- [OPTIONS]");
                println!("Options:");
                println!("  -g, --games <N>      Number of tournament games (default: 20)");
                println!("  -d, --depth <D>      Nash Engine search depth (default: 6)");
                println!("      --nic-depth <D>  NIC Replica search depth (default: 3)");
                println!("  -s, --size <S>       Board size (default: 11)");
                println!("  -v, --verbose        Enable verbose move output");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    println!("================================================================================");
    println!("        HEX ARENA: NASH ENGINE VS NINTENDO IMPOSSIBLE COMPUTER (NIC)");
    println!("================================================================================");
    println!("  Board Size       : {}x{}", board_size, board_size);
    println!("  Tournament Games : {}", num_games);
    println!("  Nash Depth       : {}", search_depth);
    println!("  NIC Depth        : {}", nic_depth);
    println!("================================================================================\n");

    let openings = [
        Some((5, 5)), // F6 (Center)
        Some((4, 4)), // E5 (Diagonal)
        Some((8, 2)), // C9 (Flank)
        Some((3, 3)), // D4 (Off-center)
        Some((4, 7)), // H5 (East flank)
        Some((6, 4)), // E7 (South center)
        Some((1, 4)), // E2 (North flank)
        Some((6, 6)), // G7 (South diagonal)
        None,         // Standard empty start
    ];

    let mut nash_wins = 0usize;
    let mut nic_wins = 0usize;
    let mut nash_blue_wins = 0usize;
    let mut nash_blue_games = 0usize;
    let mut nash_red_wins = 0usize;
    let mut nash_red_games = 0usize;
    let mut total_duration_ms = 0u128;
    let mut lost_matches: Vec<MatchResult> = Vec::new();

    let tournament_start = Instant::now();

    for m in 1..=num_games {
        // Alternate colors
        let nash_color = if m % 2 == 1 { BLUE } else { RED };
        let opening = openings[(m - 1) % openings.len()];

        let nash_name = if nash_color == BLUE { "Nash (Blue)" } else { "Nash (Red)" };
        let nic_name = if nash_color == BLUE { "NIC (Red)" } else { "NIC (Blue)" };

        println!("Match {:2}/{}: {} vs {} [Opening: {:?}]", m, num_games, nash_name, nic_name, opening);

        let res = run_single_game(m, board_size, nash_color, search_depth, nic_depth, opening, verbose);
        total_duration_ms += res.duration_ms;

        let won = res.winner == nash_color;
        if won {
            nash_wins += 1;
            if nash_color == BLUE {
                nash_blue_wins += 1;
            } else {
                nash_red_wins += 1;
            }
            println!(
                "   -> WIN for Nash Engine in {} plies ({:.2}s)\n",
                res.total_plies,
                res.duration_ms as f64 / 1000.0
            );
        } else {
            nic_wins += 1;
            println!(
                "   -> WIN for NIC Replica in {} plies ({:.2}s)\n",
                res.total_plies,
                res.duration_ms as f64 / 1000.0
            );
            lost_matches.push(res);
        }

        if nash_color == BLUE {
            nash_blue_games += 1;
        } else {
            nash_red_games += 1;
        }
    }

    let elapsed = tournament_start.elapsed();
    let win_rate = if num_games > 0 { (nash_wins as f64 / num_games as f64) * 100.0 } else { 0.0 };
    let blue_win_rate = if nash_blue_games > 0 { (nash_blue_wins as f64 / nash_blue_games as f64) * 100.0 } else { 0.0 };
    let red_win_rate = if nash_red_games > 0 { (nash_red_wins as f64 / nash_red_games as f64) * 100.0 } else { 0.0 };

    println!("================================================================================");
    println!("                           TOURNAMENT FINAL RESULTS");
    println!("================================================================================");
    println!("  Total Matches Played     : {}", num_games);
    println!("  Nash Engine Total Wins   : {} ({:.1}%)", nash_wins, win_rate);
    println!("  NIC Replica Total Wins   : {} ({:.1}%)", nic_wins, 100.0 - win_rate);
    println!("  Nash Win Rate as BLUE    : {}/{} ({:.1}%)", nash_blue_wins, nash_blue_games, blue_win_rate);
    println!("  Nash Win Rate as RED     : {}/{} ({:.1}%)", nash_red_wins, nash_red_games, red_win_rate);
    println!("  Total Tournament Time    : {:.2}s (avg {:.2}s/game)", elapsed.as_secs_f64(), total_duration_ms as f64 / (num_games as f64 * 1000.0));
    println!("================================================================================\n");

    if !lost_matches.is_empty() {
        println!("=== TACTICAL DIVERGENCE / BLUNDER REPORT ({} Losses) ===", lost_matches.len());
        for (idx, loss) in lost_matches.iter().enumerate() {
            println!("\n--- Loss #{}: Match {} (Nash as {}, Opening {}) ---", idx + 1, loss.match_num, if loss.nash_player == BLUE { "BLUE" } else { "RED" }, loss.opening_move);
            println!("{}", loss.pgn);
        }
    } else {
        println!(">>> PERFECT TOURNAMENT: ZERO LOSSES AGAINST NIC REPLICA! <<<");
    }
}
