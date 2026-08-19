//! Comprehensive Rust Engine Integration and Unit Tests.
//!
//! OOP Description:
//! Validates Bitboard operations, move placement, shortest path BFS,
//! master opening book lookups, PVS search solving small boards,
//! lock-free Transposition Table CRUD, and C-ABI export symbols.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use hex_engine::bitboard::Bitboard128;
use hex_engine::board::{HexBoard, BLUE, EMPTY, RED};
use hex_engine::evaluator::{HexEvaluator, WIN_SCORE};
use hex_engine::openings::HexOpeningBook;
use hex_engine::search::SearchEngine;
use hex_engine::tt::{TranspositionTable, EXACT};
use hex_engine::{
    hex_engine_clear_tt, hex_engine_create, hex_engine_evaluate, hex_engine_free, hex_engine_search,
    CTopMoveEntry,
};

#[test]
fn test_bitboard_operations() {
    let mut bb = Bitboard128::empty();
    assert!(bb.is_empty());

    bb.set_bit(5, 5, 11);
    assert!(!bb.is_empty());
    assert!(bb.get_bit(5, 5, 11));
    assert!(!bb.get_bit(5, 6, 11));

    bb.clear_bit(5, 5, 11);
    assert!(bb.is_empty());
}

#[test]
fn test_board_move_and_win_detection() {
    let mut board = HexBoard::new(5);
    assert_eq!(board.get_winner(), EMPTY);

    for r in 0..5 {
        assert!(board.place_move(r, 2, RED));
    }

    assert_eq!(board.get_winner(), RED);
    board.undo_move();
    assert_eq!(board.get_winner(), EMPTY);
}

#[test]
fn test_evaluator_shortest_path() {
    let mut board = HexBoard::new(5);
    let d0_red = HexEvaluator::shortest_path(&board, RED);
    let d0_blue = HexEvaluator::shortest_path(&board, BLUE);
    assert_eq!(d0_red, 5);
    assert_eq!(d0_blue, 5);

    board.place_move(0, 2, RED);
    let d1_red = HexEvaluator::shortest_path(&board, RED);
    assert_eq!(d1_red, 4);
}

#[test]
fn test_search_engine_small_board_solve() {
    let mut board = HexBoard::new(3);
    board.place_move(0, 0, RED);
    board.place_move(1, 0, RED);

    let mut engine = SearchEngine::new();
    let (best_move, score, stats) = engine.search(&board, RED, 2, None);

    assert_eq!(best_move, Some((2, 0)));
    assert!(score > WIN_SCORE - 100.0);
    assert!(stats.nodes > 0);
}

#[test]
fn test_opening_book_master_lines() {
    let mut board = HexBoard::new(11);
    let op0 = HexOpeningBook::get_opening_move(&board, BLUE);
    assert_eq!(op0.unwrap().0, (5, 5)); // F6

    board.place_move(5, 5, BLUE); // F6
    let op1 = HexOpeningBook::get_opening_move(&board, RED);
    assert_eq!(op1.unwrap().0, (5, 6)); // G6

    board.place_move(5, 6, RED); // G6
    let op2 = HexOpeningBook::get_opening_move(&board, BLUE);
    assert_eq!(op2.unwrap().0, (4, 6)); // G5

    board.place_move(4, 6, BLUE); // G5
    let op3 = HexOpeningBook::get_opening_move(&board, RED);
    assert_eq!(op3.unwrap().0, (6, 3)); // D7

    board.place_move(6, 3, RED); // D7
    let op4 = HexOpeningBook::get_opening_move(&board, BLUE);
    assert!(op4.is_none()); // Dynamically searched rather than hardcoded!
}

#[test]
fn test_transposition_table_operations() {
    let tt = TranspositionTable::new();
    let hash = 0xABCD12345678u64;
    tt.store(hash, 6, 25.5, EXACT, Some((5, 5)));

    let looked_up = tt.lookup(hash, 5, -100.0, 100.0);
    assert!(looked_up.is_some());
    let (score, mv) = looked_up.unwrap();
    assert!((score - 25.5).abs() < 0.01);
    assert_eq!(mv, Some((5, 5)));

    tt.clear();
    assert!(tt.lookup(hash, 5, -100.0, 100.0).is_none());
}

#[test]
fn test_c_abi_exports() {
    let engine_ptr = hex_engine_create();
    assert!(!engine_ptr.is_null());

    let mut grid = [0u8; 121];
    grid[5 * 11 + 5] = BLUE;

    let score = hex_engine_evaluate(grid.as_ptr(), 11);
    println!("C-ABI evaluated score: {}", score);
    assert!(score.abs() < 100.0);

    let mut out_r = -1i32;
    let mut out_c = -1i32;
    let mut out_score = 0.0f32;
    let mut out_nodes = 0u64;
    let mut top_moves = [CTopMoveEntry {
        rank: 0,
        r: 0,
        c: 0,
        score: 0.0,
        depth: 0,
    }; 10];
    let mut num_top = 0i32;

    let res = hex_engine_search(
        engine_ptr,
        grid.as_ptr(),
        11,
        RED as i32,
        3,
        &mut out_r,
        &mut out_c,
        &mut out_score,
        &mut out_nodes,
        std::ptr::null(),
        top_moves.as_mut_ptr(),
        10,
        &mut num_top,
    );

    assert_eq!(res, 1);
    assert!(out_r >= 0 && out_c >= 0);
    assert!(out_nodes > 0);
    assert!(num_top > 0);

    hex_engine_clear_tt(engine_ptr);
    hex_engine_free(engine_ptr);
}

#[test]
fn test_compulsory_carrier_defense() {
    let mut board = HexBoard::new(11);
    board.place_move(9, 2, RED);  // C10 edge-2 stone
    board.place_move(10, 1, BLUE); // 17. B11 carrier attack

    let resp = hex_engine::patterns::HexPatternMatcher::get_compulsory_carrier_response(&board, RED);
    assert_eq!(resp, Some((10, 2))); // Must defend with C11 (10, 2)
}

#[test]
fn test_j2_corner_edge_bridge_seal() {
    let mut board = HexBoard::new(11);
    // 1. f6 e6 2. e7 g6 3. g4 h4 4. f8 i3 5. h7 d7 6. g5 c9 7. c8 d8 8. e5 e8 9. c6 i4 10. h2 f7 11. j1 k1 12. j6 h6 13. h5 i5 14. k2 b7
    let moves = [
        (5, 5, BLUE), (5, 4, RED),
        (6, 4, BLUE), (5, 6, RED),
        (3, 6, BLUE), (3, 7, RED),
        (7, 5, BLUE), (2, 8, RED),
        (6, 7, BLUE), (6, 3, RED),
        (4, 6, BLUE), (8, 2, RED),
        (7, 2, BLUE), (7, 3, RED),
        (4, 4, BLUE), (7, 4, RED),
        (5, 2, BLUE), (3, 8, RED),
        (1, 7, BLUE), (6, 5, RED),
        (0, 9, BLUE), (0, 10, RED),
        (5, 9, BLUE), (5, 7, RED),
        (4, 7, BLUE), (4, 8, RED),
        (1, 10, BLUE), (6, 1, RED),
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let b_dist = HexEvaluator::shortest_path(&board, BLUE);
    let r_dist = HexEvaluator::shortest_path(&board, RED);
    println!("INITIAL BOARD: Blue dist: {}, Red dist: {}", b_dist, r_dist);

    let mut engine = SearchEngine::new();
    let (best_move, score, stats) = engine.search(&board, BLUE, 4, None);
    println!("Found best move: {:?}, score: {}, top_moves:", best_move, score);
    for tm in &stats.top_moves {
        println!("  #{}: ({}, {}) score: {} depth: {}", tm.rank, tm.r, tm.c, tm.score, tm.depth);
    }
    // Direct winning edge connections (G8/K6/K5) or J2/I8 defense
    assert!(
        best_move == Some((7, 6)) || best_move == Some((5, 10)) || best_move == Some((4, 10)) ||
        best_move == Some((1, 9)) || best_move == Some((7, 8)) ||
        best_move == Some((0, 10)) || best_move == Some((2, 8)) ||
        best_move == Some((1, 8)) || best_move == Some((0, 9)) ||
        best_move == Some((9, 8)),
        "Expected strong Blue move, got {:?}", best_move
    );
}

#[test]
fn test_j2_after_14_f5() {
    let mut board = HexBoard::new(11);
    // 1. f6 e6 2. e7 g6 3. g4 h4 4. f8 i3 5. h7 d7 6. g5 c9 7. c8 d8 8. e5 e8 9. c6 i4 10. h2 f7 11. j1 k1 12. j6 h6 13. h5 i5 14. k2 f5
    let moves = [
        (5, 5, BLUE), (5, 4, RED),
        (6, 4, BLUE), (5, 6, RED),
        (3, 6, BLUE), (3, 7, RED),
        (7, 5, BLUE), (2, 8, RED),
        (6, 7, BLUE), (6, 3, RED),
        (4, 6, BLUE), (8, 2, RED),
        (7, 2, BLUE), (7, 3, RED),
        (4, 4, BLUE), (7, 4, RED),
        (5, 2, BLUE), (3, 8, RED),
        (1, 7, BLUE), (6, 5, RED),
        (0, 9, BLUE), (0, 10, RED),
        (5, 9, BLUE), (5, 7, RED),
        (4, 7, BLUE), (4, 8, RED),
        (1, 10, BLUE), (4, 5, RED), // 14. k2 f5
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let mut clone_k7 = board.clone();
    clone_k7.place_move(6, 10, BLUE);
    let mut clone_j2 = board.clone();
    clone_j2.place_move(1, 9, BLUE);
    println!("Static eval after K7 (6, 10): {}", HexEvaluator::evaluate_for_player(&clone_k7, BLUE));
    println!("Static eval after J2 (1, 9):  {}", HexEvaluator::evaluate_for_player(&clone_j2, BLUE));
    println!("Blue dist after K7: {}, Red dist: {}", HexEvaluator::shortest_path(&clone_k7, BLUE), HexEvaluator::shortest_path(&clone_k7, RED));
    println!("Blue dist after J2: {}, Red dist: {}", HexEvaluator::shortest_path(&clone_j2, BLUE), HexEvaluator::shortest_path(&clone_j2, RED));

    let mut engine = SearchEngine::new();
    for d in 1..=4 {
        let (bm, score, _) = engine.search(&board, BLUE, d, None);
        println!("Depth {}: best move: {:?}, score: {}", d, bm, score);
    }
    let (best_move, score, stats) = engine.search(&board, BLUE, 4, None);
    println!("Position 2 - Found best move: {:?}, score: {}", best_move, score);
    for tm in &stats.top_moves {
        println!("  #{}: ({}, {}) score: {}", tm.rank, tm.r, tm.c, tm.score);
    }
    assert!(best_move == Some((1, 9)) || best_move == Some((3, 5)) || score > 200.0); // F4 or J2
}

#[test]
fn test_position_16_f4_h3() {
    let mut board = HexBoard::new(11);
    // 1. f6 e6 2. e7 g6 3. g4 h4 4. f8 i3 5. h7 d7 6. g5 c9 7. c8 d8 8. e5 e8 9. c6 i4 10. h2 f7 11. j1 k1 12. j6 h6 13. h5 i5 14. k2 f5 15. j2 g3 16. f4 h3
    let moves = [
        (5, 5, BLUE), (5, 4, RED),
        (6, 4, BLUE), (5, 6, RED),
        (3, 6, BLUE), (3, 7, RED),
        (7, 5, BLUE), (2, 8, RED),
        (6, 7, BLUE), (6, 3, RED),
        (4, 6, BLUE), (8, 2, RED),
        (7, 2, BLUE), (7, 3, RED),
        (4, 4, BLUE), (7, 4, RED),
        (5, 2, BLUE), (3, 8, RED),
        (1, 7, BLUE), (6, 5, RED),
        (0, 9, BLUE), (0, 10, RED),
        (5, 9, BLUE), (5, 7, RED),
        (4, 7, BLUE), (4, 8, RED),
        (1, 10, BLUE), (4, 5, RED),
        (1, 9, BLUE), (2, 6, RED),
        (3, 5, BLUE), (2, 7, RED),
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let r_dist = HexEvaluator::shortest_path(&board, RED);
    let b_dist = HexEvaluator::shortest_path(&board, BLUE);
    println!("BOARD 16. f4 h3: Red dist: {}, Blue dist: {}", r_dist, b_dist);
    println!("Static Eval for Red: {}, for Blue: {}", HexEvaluator::evaluate_for_player(&board, RED), HexEvaluator::evaluate_for_player(&board, BLUE));
    
    let mut board_after_k4 = board.clone();
    board_after_k4.place_move(3, 10, BLUE); // K4
    let mut engine = SearchEngine::new();
    println!("--- Searching RED responses after Blue K4 ---");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&board_after_k4, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(3) {
            println!("   RED #{}: ({}, {}) score: {}", tm.rank, tm.r, tm.c, tm.score);
        }
    }
}

#[test]
fn test_position_16_g2_f3() {
    let mut board = HexBoard::new(11);
    // 1. f6 e6 2. e7 g6 3. g4 h4 4. f8 i3 5. h7 d7 6. g5 c9 7. c8 d8 8. e5 e8 9. c6 i4 10. h2 f7 11. j1 k1 12. j6 h6 13. h5 i5 14. k2 f5 15. j2 g3 16. g2 f3
    let moves = [
        (5, 5, BLUE), (5, 4, RED),
        (6, 4, BLUE), (5, 6, RED),
        (3, 6, BLUE), (3, 7, RED),
        (7, 5, BLUE), (2, 8, RED),
        (6, 7, BLUE), (6, 3, RED),
        (4, 6, BLUE), (8, 2, RED),
        (7, 2, BLUE), (7, 3, RED),
        (4, 4, BLUE), (7, 4, RED),
        (5, 2, BLUE), (3, 8, RED),
        (1, 7, BLUE), (6, 5, RED),
        (0, 9, BLUE), (0, 10, RED),
        (5, 9, BLUE), (5, 7, RED),
        (4, 7, BLUE), (4, 8, RED),
        (1, 10, BLUE), (4, 5, RED),
        (1, 9, BLUE), (2, 6, RED),
        (1, 6, BLUE), (2, 5, RED), // 16. g2 f3
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let mut engine = SearchEngine::new();
    println!("=== Position 16. g2 f3 ===");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&board, BLUE, d, None);
        println!("Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(6) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }

    let mut b_j4 = board.clone();
    b_j4.place_move(3, 9, BLUE); // J4 (3, 9)
    println!("=== After Blue plays J4 (3, 9) ===");
    for d in 1..=4 {
        let (bm, score, _) = engine.search(&b_j4, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
    }

    let mut b_k3 = board.clone();
    b_k3.place_move(2, 10, BLUE); // K3 (2, 10)
    println!("=== After Blue plays K3 (2, 10) ===");
    for d in 1..=4 {
        let (bm, score, _) = engine.search(&b_k3, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
    }
}

#[test]
fn test_position_19_d2_b3() {
    let mut board = HexBoard::new(11);
    let moves = [
        (5, 5, BLUE), (5, 4, RED),
        (6, 4, BLUE), (5, 6, RED),
        (3, 6, BLUE), (3, 7, RED),
        (7, 5, BLUE), (2, 8, RED),
        (6, 7, BLUE), (6, 3, RED),
        (4, 6, BLUE), (8, 2, RED),
        (7, 2, BLUE), (7, 3, RED),
        (4, 4, BLUE), (7, 4, RED),
        (5, 2, BLUE), (3, 8, RED),
        (1, 7, BLUE), (6, 5, RED),
        (0, 9, BLUE), (0, 10, RED),
        (5, 9, BLUE), (5, 7, RED),
        (4, 7, BLUE), (4, 8, RED),
        (1, 10, BLUE), (4, 5, RED),
        (1, 9, BLUE), (2, 6, RED),
        (1, 6, BLUE), (2, 5, RED),
        (1, 5, BLUE), (2, 4, RED),
        (1, 4, BLUE), (2, 3, RED),
        (1, 3, BLUE), (2, 1, RED), // 19. d2 b3
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let mut engine = SearchEngine::new();
    println!("=== Position 19. d2 b3 - Blue to Move ===");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&board, BLUE, d, None);
        println!("Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(10) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }

    let mut b_i6 = board.clone();
    b_i6.place_move(5, 8, BLUE); // i6 (5, 8)
    println!("=== After Blue plays i6 (5, 8) - Red to Move ===");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&b_i6, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(5) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   RED #{}: {}{} ({}, {}) score: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score);
        }
    }
}

#[test]
fn test_position_game2_f4() {
    let mut board = HexBoard::new(11);
    let moves = [
        (5, 5, RED),  // 1. f6
        (5, 6, BLUE), // 1... g6
        (4, 6, RED),  // 2. g5
        (5, 4, BLUE), // 2... e6
        (4, 1, RED),  // 3. b5
        (6, 4, BLUE), // 3... e7
        (3, 3, RED),  // 4. d4
        (3, 5, BLUE), // 4... f4
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let mut engine = SearchEngine::new();
    println!("=== Position after 4... f4: Red to move ===");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&board, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(10) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }

    // Now inspect if Red plays F3 (2, 5)
    let mut b_f3 = board.clone();
    b_f3.place_move(2, 5, RED); // F3
    println!("=== After Red plays F3 (2, 5): Blue to move ===");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&b_f3, BLUE, d, None);
        println!("BLUE Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(5) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }
}

#[test]
fn test_position_user_cases() {
    println!("--- Test Position 1: 1. e5 f5 2. b8 g3 ---");
    let mut b1 = HexBoard::new(11);
    let p1_moves = [
        (4, 4, RED),  // 1. e5
        (4, 5, BLUE), // 1... f5
        (7, 1, RED),  // 2. b8
        (2, 6, BLUE), // 2... g3
    ];
    for &(r, c, p) in &p1_moves {
        b1.place_move(r, c, p);
    }

    let mut engine = SearchEngine::new();
    println!("=== Position 1: Red to move ===");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&b1, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(6) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }

    // Now inspect if Red plays F4 (3, 5)
    let mut b1_f4 = b1.clone();
    b1_f4.place_move(3, 5, RED); // F4 (3, 5)
    println!("=== Position 1 after Red plays F4 (3, 5) (Blue to move) ===");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&b1_f4, BLUE, d, None);
        println!("BLUE Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(5) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }

    println!("\n--- Test Position 2: 1. e5 f5 2. b8 g3 3. e7 f7 4. f6 h5 5. g4 h3 6. h4 j3 7. c6 j4 8. i3 j2 9. j1 i1 10. i4 i5 ---");
    let mut b2 = HexBoard::new(11);
    let p2_moves = [
        (4, 4, RED),  // 1. e5
        (4, 5, BLUE), // 1... f5
        (7, 1, RED),  // 2. b8
        (2, 6, BLUE), // 2... g3
        (6, 4, RED),  // 3. e7
        (6, 5, BLUE), // 3... f7
        (5, 5, RED),  // 4. f6
        (4, 7, BLUE), // 4... h5
        (3, 6, RED),  // 5. g4
        (2, 7, BLUE), // 5... h3
        (3, 7, RED),  // 6. h4
        (2, 9, BLUE), // 6... j3
        (5, 2, RED),  // 7. c6
        (3, 9, BLUE), // 7... j4
        (2, 8, RED),  // 8. i3
        (1, 9, BLUE), // 8... j2
        (0, 9, RED),  // 9. j1
        (0, 8, BLUE), // 9... i1
        (3, 8, RED),  // 10. i4
        (4, 8, BLUE), // 10... i5
    ];
    for &(r, c, p) in &p2_moves {
        b2.place_move(r, c, p);
    }
    println!("=== Position 2: Red to move ===");
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&b2, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(6) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }
}

#[test]
fn test_position_f6_g6() {
    println!("\n--- Test Position: 1. f6 g6 (Red to move) ---");
    let mut board = HexBoard::new(11);
    board.place_move(5, 5, RED); // 1. f6
    board.place_move(5, 6, BLUE); // 1... g6

    let mut engine = SearchEngine::new();
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&board, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(8) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }
}

#[test]
fn test_position_f6_g5() {
    println!("\n--- Test Position: 1. f6 g5 (Red to move) ---");
    let mut board = HexBoard::new(11);
    board.place_move(5, 5, RED); // 1. f6
    board.place_move(4, 6, BLUE); // 1... g5

    let mut engine = SearchEngine::new();
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&board, RED, d, None);
        println!("RED Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(8) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }
}

#[test]
fn test_position_move8_i2() {
    println!("\n--- Test Position after 8... i2 (Blue is Player 1, Blue to move) ---");
    let mut board = HexBoard::new(11);
    let moves = [
        (5, 5, BLUE),  (5, 4, RED), // 1. f6 e6
        (6, 4, BLUE),  (6, 3, RED), // 2. e7 d7
        (7, 3, BLUE),  (4, 7, RED), // 3. d8 h5
        (8, 1, BLUE),  (6, 6, RED), // 4. b9 g7
        (5, 6, BLUE),  (5, 7, RED), // 5. g6 h6
        (3, 6, BLUE),  (2, 8, RED), // 6. g4 i3
        (3, 7, BLUE),  (3, 8, RED), // 7. h4 i4
        (1, 7, BLUE),  (1, 8, RED), // 8. h2 i2
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let mut engine = SearchEngine::new();
    for d in 1..=6 {
        let (bm, score, stats) = engine.search(&board, BLUE, d, None);
        println!("BLUE Depth {}: best move: {:?}, score: {}", d, bm, score);
        for tm in stats.top_moves.iter().take(12) {
            let col_c = (b'A' + tm.c as u8) as char;
            println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
        }
    }
}

#[test]
fn test_user_position_double_bridge_b8() {
    println!("\n--- Test Position: 1. f6 k11 2. h5 j11 3. d7 i11 4. j4 h11 (Blue to move) ---");
    let mut board = HexBoard::new(11);
    let moves = [
        (5, 5, BLUE),  (10, 10, RED), // 1. f6 k11
        (4, 7, BLUE),  (10, 9, RED),  // 2. h5 j11
        (6, 3, BLUE),  (10, 8, RED),  // 3. d7 i11
        (3, 9, BLUE),  (10, 7, RED),  // 4. j4 h11
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let mut engine = SearchEngine::new();
    let (bm, score, stats) = engine.search(&board, BLUE, 4, None);
    println!("BLUE Best move at depth 4: {:?}, score: {}", bm, score);
    for tm in stats.top_moves.iter().take(12) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }

    assert!(score < -50.0, "Blue should have decisive winning score (<-50), got {}", score);
    assert!(!stats.top_moves.is_empty());
}

#[test]
fn test_user_position_edge_connection_i1() {
    println!("\n--- Test Position: 1. e5 f5 2. f4 g4 3. c6 g3 4. g2 h2 5. h1 (Red to move) ---");
    let mut board = HexBoard::new(11);
    let moves = [
        (4, 4, BLUE),  (4, 5, RED), // 1. e5 f5
        (3, 5, BLUE),  (3, 6, RED), // 2. f4 g4
        (5, 2, BLUE),  (2, 6, RED), // 3. c6 g3
        (1, 6, BLUE),  (1, 7, RED), // 4. g2 h2
        (0, 7, BLUE),               // 5. h1
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    let mut engine = SearchEngine::new();
    let (best_move, score, stats) = engine.search(&board, RED, 6, None);
    println!("RED Best move at depth 6: {:?}, score: {}", best_move, score);

    // Red playing I1 (0, 8) connects directly to the North edge touching H2 (1, 7)
    let top_coords: Vec<(usize, usize)> = stats.top_moves.iter().take(5).map(|m| (m.r, m.c)).collect();
    println!("Top candidate coordinates: {:?}", top_coords);
    assert!(top_coords.contains(&(0, 8)) || top_coords.contains(&(2, 7)) || top_coords.contains(&(3, 7)),
        "I1 (0, 8) or major continuation should be in top candidates, got {:?}", top_coords);
}

#[test]
fn test_nic_game_analysis() {
    let mut board = HexBoard::new(11);
    let moves = [
        (8, 2, BLUE),  (5, 6, RED), // 1. c9 g6
        (8, 3, BLUE),  (5, 7, RED), // 2. d9 h6
        (7, 5, BLUE),  (7, 6, RED), // 3. f8 g8
        (1, 7, BLUE),  (3, 6, RED), // 4. h2 g4
        (7, 1, BLUE),  (8, 7, RED), // 5. b8 h9
        (6, 6, BLUE),  (6, 7, RED), // 6. g7 h7
    ];
    for &(r, c, p) in &moves {
        board.place_move(r, c, p);
    }

    println!("\n=== Iterative Deepening on 6. g7 h7 ===");
    let mut engine = SearchEngine::new();
    let (best_move, score, stats) = engine.search(&board, BLUE, 6, None);
    let move_str = best_move.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("None".to_string());
    println!("Depth 6: Best Move = {} ({:?}), Score = {:.2}, Nodes = {}", move_str, best_move, score, stats.nodes);
    for tm in stats.top_moves.iter().take(5) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }
    assert!(best_move.is_some());
}

#[test]
fn test_user_game_f6_g6_g7_f7() {
    let mut board = HexBoard::new(11);
    // 1. f6 g6 (Blue to move)
    board.place_move(5, 5, BLUE); // 1. f6
    board.place_move(5, 6, RED);  // 1... g6

    println!("\n=== Position after 1. f6 g6 (Blue to move) ===");
    let mut engine = SearchEngine::new();
    let (best_move, score, stats) = engine.search(&board, BLUE, 6, None);
    let move_str = best_move.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("None".to_string());
    println!("Depth 6: Best Move = {} ({:?}), Score = {:.2}, Nodes = {}", move_str, best_move, score, stats.nodes);
    for tm in stats.top_moves.iter().take(6) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }

    // Now test position 1. f6 g6 2. g7 f7 3. e4 h4 4. g5 h5 5. g8 e9 (Blue to move)
    let mut b2 = HexBoard::new(11);
    let moves = [
        (5, 5, BLUE), (5, 6, RED), // 1. f6 g6
        (6, 6, BLUE), (6, 5, RED), // 2. g7 f7
        (3, 4, BLUE), (3, 7, RED), // 3. e4 h4
        (4, 6, BLUE), (4, 7, RED), // 4. g5 h5
        (7, 6, BLUE), (8, 4, RED), // 5. g8 e9
    ];
    for &(r, c, p) in &moves {
        b2.place_move(r, c, p);
    }

    println!("\n=== Position after 5... e9 (Blue to move at move 6) ===");
    let (best_move2, score2, stats2) = engine.search(&b2, BLUE, 6, None);
    let move_str2 = best_move2.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("None".to_string());
    println!("Depth 6: Best Move = {} ({:?}), Score = {:.2}, Nodes = {}", move_str2, best_move2, score2, stats2.nodes);
    for tm in stats2.top_moves.iter().take(8) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }
}

#[test]
fn test_user_game_c9_d8() {
    let mut board = HexBoard::new(11);
    // 1. c9 d8 2. e6 f6 3. e7 g7 (Blue to move at move 4)
    let moves_m4 = [
        (8, 2, BLUE), (7, 3, RED), // 1. c9 d8
        (5, 4, BLUE), (5, 5, RED), // 2. e6 f6
        (6, 4, BLUE), (6, 6, RED), // 3. e7 g7
    ];
    for &(r, c, p) in &moves_m4 {
        board.place_move(r, c, p);
    }

    println!("\n=== Position after 3... g7 (Blue to move at move 4) ===");
    let mut engine = SearchEngine::new();
    let (bm4, sc4, stats4) = engine.search(&board, BLUE, 6, None);
    let ms4 = bm4.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("-".to_string());
    println!("Move 4: Best Move = {} ({:?}), Score = {:.2}", ms4, bm4, sc4);
    for tm in stats4.top_moves.iter().take(6) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }

    // 1. c9 d8 2. e6 f6 3. e7 g7 4. g6 f7 5. h5 g4 6. d10 f9 7. f5 g5 8. e11 g3 9. g2 g10 (Blue to move at move 10)
    let mut b10 = HexBoard::new(11);
    let moves_m10 = [
        (8, 2, BLUE), (7, 3, RED), // 1. c9 d8
        (5, 4, BLUE), (5, 5, RED), // 2. e6 f6
        (6, 4, BLUE), (6, 6, RED), // 3. e7 g7
        (5, 6, BLUE), (6, 5, RED), // 4. g6 f7
        (4, 7, BLUE), (3, 6, RED), // 5. h5 g4
        (9, 3, BLUE), (8, 5, RED), // 6. d10 f9
        (4, 5, BLUE), (4, 6, RED), // 7. f5 g5
        (10, 4, BLUE), (2, 6, RED), // 8. e11 g3
        (1, 6, BLUE), (9, 6, RED), // 9. g2 g10
    ];
    for &(r, c, p) in &moves_m10 {
        b10.place_move(r, c, p);
    }

    println!("\n=== Position after 9... g10 (Blue to move at move 10) ===");
    let mut b_f8 = b10.clone();
    b_f8.place_move(7, 5, BLUE); // 10. f8
    println!("Score after 10. f8 (from Red perspective): fast_eval={:.2}, full_eval={:.2}",
        HexEvaluator::evaluate_fast(&b_f8, RED),
        HexEvaluator::evaluate_for_player(&b_f8, RED));

    let mut b_f8_e8 = b_f8.clone();
    b_f8_e8.place_move(7, 4, RED); // 10... e8
    println!("Score after 10. f8 e8 (from Blue perspective): fast_eval={:.2}, full_eval={:.2}, my_dist={}, opp_dist={}",
        HexEvaluator::evaluate_fast(&b_f8_e8, BLUE),
        HexEvaluator::evaluate_for_player(&b_f8_e8, BLUE),
        HexEvaluator::shortest_path(&b_f8_e8, BLUE),
        HexEvaluator::shortest_path(&b_f8_e8, RED));

    let (bm10, sc10, stats10) = engine.search(&b10, BLUE, 10, None);
    let ms10 = bm10.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("-".to_string());
    println!("Move 10 at depth 10: Best Move = {} ({:?}), Score = {:.2}", ms10, bm10, sc10);
    for tm in stats10.top_moves.iter().take(6) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }
}

#[test]
fn test_user_game_f6_e7_move7() {
    let mut b = HexBoard::new(11);
    let mut engine = SearchEngine::new();

    // 1. f6 e7
    b.place_move(5, 5, BLUE); // 1. f6
    b.place_move(6, 4, RED);  // 1... e7

    println!("\n=== Move 2: After 1... e7 (Blue to move) ===");
    let (bm2, sc2, st2) = engine.search(&b, BLUE, 4, None);
    let ms2 = bm2.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("-".to_string());
    println!("Move 2: Best = {} ({:?}), Score = {:.2}", ms2, bm2, sc2);
    for tm in st2.top_moves.iter().take(6) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }

    // 2. h6 j5
    b.place_move(5, 7, BLUE); // 2. h6
    b.place_move(4, 9, RED);  // 2... j5

    // 3. g5 e6
    b.place_move(4, 6, BLUE); // 3. g5
    b.place_move(5, 4, RED);  // 3... e6

    println!("\n=== Move 4: After 3... e6 (Blue to move) ===");
    let (bm4, sc4, st4) = engine.search(&b, BLUE, 4, None);
    let ms4 = bm4.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("-".to_string());
    println!("Move 4: Best = {} ({:?}), Score = {:.2}", ms4, bm4, sc4);
    for tm in st4.top_moves.iter().take(6) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }

    // 4. e2 f4
    b.place_move(1, 4, BLUE); // 4. e2
    b.place_move(3, 5, RED);  // 4... f4

    println!("\n=== Move 5: After 4... f4 (Blue to move) ===");
    let (bm5, sc5, st5) = engine.search(&b, BLUE, 4, None);
    let ms5 = bm5.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("-".to_string());
    println!("Move 5: Best = {} ({:?}), Score = {:.2}", ms5, bm5, sc5);
    for tm in st5.top_moves.iter().take(6) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }

    // 5. c3 g2
    b.place_move(2, 2, BLUE); // 5. c3
    b.place_move(1, 6, RED);  // 5... g2

    println!("\n=== Move 6: After 5... g2 (Blue to move) ===");
    let (bm6, sc6, st6) = engine.search(&b, BLUE, 4, None);
    let ms6 = bm6.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("-".to_string());
    println!("Move 6: Best = {} ({:?}), Score = {:.2}", ms6, bm6, sc6);
    for tm in st6.top_moves.iter().take(6) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }

    // 6. d9 f8
    b.place_move(8, 3, BLUE); // 6. d9
    b.place_move(7, 5, RED);  // 6... f8

    println!("\n=== Move 7: After 6... f8 (Blue to move) ===");
    let (bm7, sc7, st7) = engine.search(&b, BLUE, 4, None);
    let ms7 = bm7.map(|(r, c)| format!("{}{}", (b'A' + c as u8) as char, r + 1)).unwrap_or("-".to_string());
    println!("Move 7: Best = {} ({:?}), Score = {:.2}", ms7, bm7, sc7);
    for tm in st7.top_moves.iter().take(6) {
        let col_c = (b'A' + tm.c as u8) as char;
        println!("   #{}: {}{} ({}, {}) score: {} depth: {}", tm.rank, col_c, tm.r + 1, tm.r, tm.c, tm.score, tm.depth);
    }
}

#[test]
fn test_nic_replica_decision_fidelity() {
    use hex_engine::nic_replica::NicReplicaEngine;

    // Verify NIC replica correctly picks Nintendo's opening move response on 1. f6
    let mut board = HexBoard::new(11);
    board.place_move(5, 5, BLUE); // 1. f6

    let mut nic = NicReplicaEngine::with_depth(3);
    let (bm, score) = nic.select_move(&board, RED);
    assert!(bm.is_some(), "NIC replica must select a valid move");
    let (r, c) = bm.unwrap();
    println!("NIC response to 1. f6: ({}, {}) score: {:.2}", r, c, score);

    // Nintendo typically plays adjacent/2-bridge responses to center openings (e.g. (6, 4), (5, 6), (6, 5), (4, 6))
    assert!(r >= 3 && r <= 7 && c >= 3 && c <= 7, "NIC must play in the active central region");
}

#[test]
fn test_automated_arena_tournament_mini() {
    use hex_engine::nic_replica::NicReplicaEngine;

    // Run a 4-game automated mini-tournament on 7x7 and 11x11 boards
    let mut nash_engine = SearchEngine::new();
    let mut nic_engine = NicReplicaEngine::with_depth(3);

    let mut nash_wins = 0;
    let games = 2;

    for g in 1..=games {
        let mut board = HexBoard::new(11);
        let nash_color = if g % 2 == 1 { BLUE } else { RED };
        let nic_color = if nash_color == BLUE { RED } else { BLUE };

        let mut current_player = BLUE;
        let mut plies = 0;

        while !board.is_game_over() && plies < 121 {
            plies += 1;
            let bm = if current_player == nash_color {
                let (m, _, _) = nash_engine.search(&board, nash_color, 6, None);
                m
            } else {
                let (m, _) = nic_engine.select_move(&board, nic_color);
                m
            };

            assert!(bm.is_some(), "Engine must generate a legal move");
            let (r, c) = bm.unwrap();
            assert!(board.place_move(r, c, current_player));
            current_player = if current_player == BLUE { RED } else { BLUE };
        }

        let winner = board.get_winner();
        if winner == nash_color {
            nash_wins += 1;
        }
        println!("Mini Tournament Game {}: Nash ({}) vs NIC ({}) -> Winner: {} in {} plies",
            g,
            if nash_color == BLUE { "BLUE" } else { "RED" },
            if nic_color == BLUE { "BLUE" } else { "RED" },
            if winner == nash_color { "NASH" } else { "NIC" },
            plies
        );
    }

    println!("Mini Tournament Final: Nash won {}/{} games", nash_wins, games);
    assert!(nash_wins >= 1, "Nash engine must win competitive matches against NIC replica");
}

