/// NIC Replica Fidelity Verification against real NIC game transcripts.
/// Also analyzes our engine's evaluation at critical positions where it chose
/// losing moves in real games.

use hex_engine::board::{HexBoard, BLUE, RED};
use hex_engine::nic_replica::NicReplicaEngine;
use hex_engine::search::SearchEngine;
use hex_engine::evaluator::HexEvaluator;
use hex_engine::resistance::ResistanceEvaluator;

fn pgn_to_rc(pgn: &str) -> (usize, usize) {
    let chars: Vec<char> = pgn.chars().collect();
    let c = (chars[0] as u8 - b'a') as usize;
    let r: usize = pgn[1..].parse::<usize>().unwrap() - 1;
    (r, c)
}

fn rc_to_pgn(r: usize, c: usize) -> String {
    let col_char = (b'a' + c as u8) as char;
    format!("{}{}", col_char, r + 1)
}

fn build_position(moves: &[&str]) -> HexBoard {
    let mut board = HexBoard::new(11);
    let mut player = BLUE;
    for mv in moves {
        let (r, c) = pgn_to_rc(mv);
        board.place_move(r, c, player);
        player = if player == BLUE { RED } else { BLUE };
    }
    board
}

/// Check what NIC plays at each position and compare with real game transcript
fn verify_nic_responses(desc: &str, moves: &[&str]) {
    println!("\n=== {} ===", desc);
    let mut board = HexBoard::new(11);
    let mut nic = NicReplicaEngine::with_depth(3);
    let mut player = BLUE;
    
    let mut match_count = 0;
    let mut total_checks = 0;
    
    for (i, mv) in moves.iter().enumerate() {
        let (r, c) = pgn_to_rc(mv);
        board.place_move(r, c, player);
        
        if i < moves.len() - 1 {
            let respond_as = if player == BLUE { RED } else { BLUE };
            let (nic_move, nic_score) = nic.select_move(&board, respond_as);
            let nic_pgn = match nic_move {
                Some((r2, c2)) => rc_to_pgn(r2, c2),
                None => "NONE".to_string(),
            };
            let expected = moves[i + 1];
            let mark = if nic_pgn == expected { "✓" } else { "✗" };
            if nic_pgn == expected { match_count += 1; }
            total_checks += 1;
            println!("  {:2}. {} {} -> NIC {} plays {} (real: {}) [sc:{:.0}] {}",
                i + 1, if player == BLUE { "B" } else { "R" }, mv,
                if respond_as == BLUE { "B" } else { "R" },
                nic_pgn, expected, nic_score, mark);
        }
        
        player = if player == BLUE { RED } else { BLUE };
    }
    println!("  Fidelity: {}/{} ({:.0}%)", match_count, total_checks, 
        if total_checks > 0 { match_count as f64 / total_checks as f64 * 100.0 } else { 0.0 });
}

/// Analyze double-bridge chain detection
fn analyze_double_bridge_chain(desc: &str, moves: &[&str]) {
    println!("\n=== DOUBLE-BRIDGE: {} ===", desc);
    let board = build_position(moves);
    
    let red_dist = HexEvaluator::shortest_path(&board, RED);
    let blue_dist = HexEvaluator::shortest_path(&board, BLUE);
    let (r_red, r_blue) = ResistanceEvaluator::compute_resistances(&board);
    let red_eval = HexEvaluator::evaluate_for_player(&board, RED);
    
    println!("  Red dist: {} | Blue dist: {} | R_red: {:.4} | R_blue: {:.4} | Red eval: {:.2}", 
        red_dist, blue_dist, r_red, r_blue, red_eval);
    
    if red_dist == 0 { println!("  >>> RED HAS VIRTUAL CONNECTION <<<"); }
    if blue_dist == 0 { println!("  >>> BLUE HAS VIRTUAL CONNECTION <<<"); }
    if red_dist > 0 && r_red < 0.5 { println!("  >>> RED NEAR-WIN (low resistance but nonzero dist) <<<"); }
}

fn main() {
    println!("=============================================");
    println!("  NIC REPLICA FIDELITY & ENGINE DIAGNOSIS");
    println!("=============================================");

    // === NIC Response Verification ===
    
    verify_nic_responses(
        "Game 1 (d7): 1. e5 f5 2. f4 g4 3. h5 e7 ...",
        &["e5", "f5", "f4", "g4", "h5", "e7", "e6", "f6", "h2", "g3", 
          "c6", "g2", "e8", "d8", "d9", "c9", "c10", "b10", "j4", "a11", "i6", "h1"]
    );
    
    verify_nic_responses(
        "Game 2 (d7): 1. f6 e6 2. g6 f7 3. e7 i5 ...",
        &["f6", "e6", "g6", "f7", "e7", "i5", "c8", "h7", "h4", "j3",
          "i2", "k1", "g9", "f8", "k2", "j2", "i4", "j4", "h6", "i6",
          "g7", "g8", "i8", "e10", "d9", "e9", "b10", "d11"]
    );
    
    verify_nic_responses(
        "Game 3 (d8): double bridge line",
        &["c9", "h4", "b11", "d8", "a10", "e9", "g6", "d10", "g7", "e6", "b8", "f4", "i5", "g2"]
    );
    
    verify_nic_responses(
        "Game 4 (d8): double bridge win",
        &["f6", "g6", "d7", "h5", "e6", "i3", "g10", "f8", "i2", "j2", "f9", "d9"]
    );
    
    verify_nic_responses(
        "Game 5 (d9): double bridge win",
        &["e5", "f5", "f4", "h4", "h3", "i3", "c7", "j2", "d5", "e7", "g6", "d9"]
    );

    // === Double-Bridge Chain Detection ===
    println!("\n\n========== DOUBLE-BRIDGE CHAIN DETECTION ==========");
    
    analyze_double_bridge_chain(
        "Game 3 final",
        &["c9", "h4", "b11", "d8", "a10", "e9", "g6", "d10", "g7", "e6", "b8", "f4", "i5", "g2"]
    );
    
    analyze_double_bridge_chain(
        "Game 4 final",
        &["f6", "g6", "d7", "h5", "e6", "i3", "g10", "f8", "i2", "j2", "f9", "d9"]
    );
    
    analyze_double_bridge_chain(
        "Game 5 final",
        &["e5", "f5", "f4", "h4", "h3", "i3", "c7", "j2", "d5", "e7", "g6", "d9"]
    );

    // === Engine Top Moves at Critical Positions ===
    println!("\n\n========== ENGINE TOP MOVES AT CRITICAL POSITIONS ==========");
    
    // Position after 1. e5 f5 2. f4 h4 - Blue to move
    {
        let board = build_position(&["e5", "f5", "f4", "h4"]);
        let mut engine = SearchEngine::new();
        let (_mv, _score, stats) = engine.search(&board, BLUE, 6, None);
        println!("\nAfter 1. e5 f5 2. f4 h4 (Blue to move, d6):");
        for entry in stats.top_moves.iter().take(5) {
            println!("  #{}: {} score={:.2} depth={}", 
                entry.rank, rc_to_pgn(entry.r, entry.c), entry.score, entry.depth);
        }
    }
    
    // Position after 1. c9 h4 2. b11 d8 3. a10 e9 - Blue to move  
    {
        let board = build_position(&["c9", "h4", "b11", "d8", "a10", "e9"]);
        let mut engine = SearchEngine::new();
        let (_mv, _score, stats) = engine.search(&board, BLUE, 6, None);
        println!("\nAfter 1. c9 h4 2. b11 d8 3. a10 e9 (Blue to move, d6):");
        for entry in stats.top_moves.iter().take(5) {
            println!("  #{}: {} score={:.2} depth={}", 
                entry.rank, rc_to_pgn(entry.r, entry.c), entry.score, entry.depth);
        }
    }
    
    // Position after 1. f6 g6 2. d7 h5 - Blue to move
    {
        let board = build_position(&["f6", "g6", "d7", "h5"]);
        let mut engine = SearchEngine::new();
        let (_mv, _score, stats) = engine.search(&board, BLUE, 6, None);
        println!("\nAfter 1. f6 g6 2. d7 h5 (Blue to move, d6):");
        for entry in stats.top_moves.iter().take(5) {
            println!("  #{}: {} score={:.2} depth={}", 
                entry.rank, rc_to_pgn(entry.r, entry.c), entry.score, entry.depth);
        }
    }
}
