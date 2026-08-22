//! C-ABI Shared Library Interface and Module Exports.
//!
//! OOP Description:
//! This module exposes high-speed C-ABI symbols (`hex_engine_*`) from the Rust core
//! for seamless dynamic linking with C++, Python ctypes, and external game GUIs.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

pub mod bitboard;
pub mod board;
pub mod evaluator;
pub mod nic_replica;
pub mod openings;
pub mod patterns;
pub mod resistance;
pub mod search;
pub mod tt;

use std::sync::atomic::{AtomicBool, AtomicU64};

use board::{HexBoard, BLUE, RED};
use evaluator::HexEvaluator;
use search::{SearchEngine, TopMoveEntry};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CTopMoveEntry {
    pub rank: i32,
    pub r: i32,
    pub c: i32,
    pub score: f32,
    pub depth: i32,
}

/// Usage:
///     let engine = hex_engine_create();
/// Usage Example:
///     let engine = hex_engine_create();
/// Description:
///     Creates and heap-allocates a new SearchEngine instance.
#[no_mangle]
pub extern "C" fn hex_engine_create() -> *mut SearchEngine {
    Box::into_raw(Box::new(SearchEngine::new()))
}

/// Usage:
///     hex_engine_destroy(engine_ptr);
/// Usage Example:
///     hex_engine_destroy(engine);
/// Description:
///     Frees a heap-allocated SearchEngine instance.
#[no_mangle]
pub extern "C" fn hex_engine_destroy(engine: *mut SearchEngine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

/// Usage:
///     hex_engine_free(engine_ptr);
/// Usage Example:
///     hex_engine_free(engine);
/// Description:
///     Alias for hex_engine_destroy.
#[no_mangle]
pub extern "C" fn hex_engine_free(engine: *mut SearchEngine) {
    hex_engine_destroy(engine);
}

/// Usage:
///     let eval = hex_engine_evaluate(board_flat, 11);
/// Usage Example:
///     let eval = hex_engine_evaluate(board, 11);
/// Description:
///     Calculates static position evaluation score from Red's perspective.
#[no_mangle]
pub extern "C" fn hex_engine_evaluate(board_flat: *const u8, size: i32) -> f32 {
    if board_flat.is_null() || size <= 0 || size > 14 {
        return 0.0;
    }

    let s = size as usize;
    let slice = unsafe { std::slice::from_raw_parts(board_flat, s * s) };
    let mut board = HexBoard::new(s);

    for r in 0..s {
        for c in 0..s {
            let p = slice[r * s + c];
            if p == RED || p == BLUE {
                board.place_move(r, c, p);
            }
        }
    }

    HexEvaluator::evaluate_absolute(&board)
}

/// Usage:
///     let winner = hex_engine_get_winner(board_flat, 11);
/// Usage Example:
///     let winner = hex_engine_get_winner(board, 11);
/// Description:
///     Detects terminal winner using SIMD bitboard flood-fill.
#[no_mangle]
pub extern "C" fn hex_engine_get_winner(board_flat: *const u8, size: i32) -> i32 {
    if board_flat.is_null() || size <= 0 || size > 14 {
        return 0;
    }

    let s = size as usize;
    let slice = unsafe { std::slice::from_raw_parts(board_flat, s * s) };
    let mut board = HexBoard::new(s);

    for r in 0..s {
        for c in 0..s {
            let p = slice[r * s + c];
            if p == RED || p == BLUE {
                board.place_move(r, c, p);
            }
        }
    }

    board.get_winner() as i32
}

/// Usage:
///     hex_engine_search(...);
/// Usage Example:
///     hex_engine_search(engine, board, 11, 2, 8, &r, &c, &score, &nodes, cancel, top, 12, &num_top);
/// Description:
///     Executes iterative deepening PVS search and populates best move, score, nodes, and top candidate moves.
#[no_mangle]
pub extern "C" fn hex_engine_search(
    engine_ptr: *mut SearchEngine,
    board_flat: *const u8,
    size: i32,
    player: i32,
    max_depth: i32,
    out_r: *mut i32,
    out_c: *mut i32,
    out_score: *mut f32,
    out_nodes: *mut u64,
    cancel_flag_ptr: *const AtomicBool,
    out_top_moves: *mut CTopMoveEntry,
    max_top: i32,
    out_num_top: *mut i32,
) -> i32 {
    if engine_ptr.is_null() || board_flat.is_null() || size <= 0 || size > 14 {
        return 0;
    }

    let engine = unsafe { &mut *engine_ptr };
    let s = size as usize;
    let slice = unsafe { std::slice::from_raw_parts(board_flat, s * s) };
    let mut board = HexBoard::new(s);

    for r in 0..s {
        for c in 0..s {
            let p = slice[r * s + c];
            if p == RED || p == BLUE {
                board.place_move(r, c, p);
            }
        }
    }

    let p = player as u8;
    board.set_current_player(p);
    let cancel_flag = if !cancel_flag_ptr.is_null() {
        Some(unsafe { &*cancel_flag_ptr })
    } else {
        None
    };

    let (best_move, score, stats) = engine.search(&board, p, max_depth as u8, cancel_flag);

    unsafe {
        if let Some((r, c)) = best_move {
            if !out_r.is_null() {
                *out_r = r as i32;
            }
            if !out_c.is_null() {
                *out_c = c as i32;
            }
        } else {
            if !out_r.is_null() {
                *out_r = -1;
            }
            if !out_c.is_null() {
                *out_c = -1;
            }
        }

        if !out_score.is_null() {
            *out_score = score;
        }
        if !out_nodes.is_null() {
            *out_nodes = stats.nodes;
        }

        if !out_top_moves.is_null() && !out_num_top.is_null() && max_top > 0 {
            let count = stats.top_moves.len().min(max_top as usize);
            let out_slice = std::slice::from_raw_parts_mut(out_top_moves, count);
            for i in 0..count {
                let tm = &stats.top_moves[i];
                out_slice[i] = CTopMoveEntry {
                    rank: tm.rank as i32,
                    r: tm.r as i32,
                    c: tm.c as i32,
                    score: tm.score,
                    depth: tm.depth as i32,
                };
            }
            *out_num_top = count as i32;
        }
    }

    1
}

/// Usage:
///     hex_engine_search_with_cancel(...);
/// Usage Example:
///     hex_engine_search_with_cancel(engine, board, 11, 2, 8, cancel, &r, &c, &score, &nodes, top, 12, &num_top);
/// Description:
///     C-ABI export matching hex_engine_search.
#[no_mangle]
pub extern "C" fn hex_engine_search_with_cancel(
    engine_ptr: *mut SearchEngine,
    board_flat: *const u8,
    size: i32,
    player: i32,
    max_depth: i32,
    cancel_flag_ptr: *const AtomicBool,
    out_r: *mut i32,
    out_c: *mut i32,
    out_score: *mut f32,
    out_nodes: *mut u64,
    out_top_moves: *mut CTopMoveEntry,
    max_top: i32,
    out_num_top: *mut i32,
) -> i32 {
    hex_engine_search(
        engine_ptr,
        board_flat,
        size,
        player,
        max_depth,
        out_r,
        out_c,
        out_score,
        out_nodes,
        cancel_flag_ptr,
        out_top_moves,
        max_top,
        out_num_top,
    )
}

pub type LiveCandidateBatchCallback = Option<
    unsafe extern "C" fn(
        user_data: *mut std::ffi::c_void,
        moves_ptr: *const CTopMoveEntry,
        num_moves: i32,
        best_r: i32,
        best_c: i32,
        best_score: f32,
    )
>;

/// Usage:
///     hex_engine_search_step(...);
/// Usage Example:
///     hex_engine_search_step(engine, board, 11, 2, 4, cancel, live_nodes, &r, &c, &score, &nodes, top, 12, &num);
/// Description:
///     Executes a single depth step with real-time live atomic node counter streaming.
#[no_mangle]
pub extern "C" fn hex_engine_search_step(
    engine_ptr: *mut SearchEngine,
    board_flat: *const u8,
    size: i32,
    player: i32,
    depth: i32,
    cancel_flag_ptr: *const AtomicBool,
    live_nodes_ptr: *const AtomicU64,
    out_r: *mut i32,
    out_c: *mut i32,
    out_score: *mut f32,
    out_nodes: *mut u64,
    out_top_moves: *mut CTopMoveEntry,
    max_top: i32,
    out_num_top: *mut i32,
) -> i32 {
    hex_engine_search_step_streaming(
        engine_ptr,
        board_flat,
        size,
        player,
        depth,
        cancel_flag_ptr,
        live_nodes_ptr,
        None,
        std::ptr::null_mut(),
        out_r,
        out_c,
        out_score,
        out_nodes,
        out_top_moves,
        max_top,
        out_num_top,
    )
}

/// Usage:
///     hex_engine_search_step_streaming(...);
/// Usage Example:
///     hex_engine_search_step_streaming(engine, board, 11, 2, 4, cancel, live_nodes, cb, user_data, &r, &c, &score, &nodes, top, 12, &num);
/// Description:
///     Executes a single depth step with real-time live candidate move streaming callback.
#[no_mangle]
pub extern "C" fn hex_engine_search_step_streaming(
    engine_ptr: *mut SearchEngine,
    board_flat: *const u8,
    size: i32,
    player: i32,
    depth: i32,
    cancel_flag_ptr: *const AtomicBool,
    live_nodes_ptr: *const AtomicU64,
    live_callback: LiveCandidateBatchCallback,
    user_data: *mut std::ffi::c_void,
    out_r: *mut i32,
    out_c: *mut i32,
    out_score: *mut f32,
    out_nodes: *mut u64,
    out_top_moves: *mut CTopMoveEntry,
    max_top: i32,
    out_num_top: *mut i32,
) -> i32 {
    if engine_ptr.is_null() || board_flat.is_null() || size <= 0 || size > 14 {
        return 0;
    }

    let engine = unsafe { &mut *engine_ptr };
    let s = size as usize;
    let slice = unsafe { std::slice::from_raw_parts(board_flat, s * s) };
    let mut board = HexBoard::new(s);

    for r in 0..s {
        for c in 0..s {
            let p = slice[r * s + c];
            if p == RED || p == BLUE {
                board.place_move(r, c, p);
            }
        }
    }

    let p = player as u8;
    board.set_current_player(p);
    let cancel_flag = if !cancel_flag_ptr.is_null() {
        Some(unsafe { &*cancel_flag_ptr })
    } else {
        None
    };
    let live_nodes = if !live_nodes_ptr.is_null() {
        Some(unsafe { &*live_nodes_ptr })
    } else {
        None
    };

    let rust_callback = live_callback.map(|cb| {
        move |entries: &[TopMoveEntry], b_move: Option<(usize, usize)>, b_score: f32| {
            let count = entries.len().min(12);
            let mut c_entries = Vec::with_capacity(count);
            for e in entries.iter().take(count) {
                c_entries.push(CTopMoveEntry {
                    rank: e.rank as i32,
                    r: e.r as i32,
                    c: e.c as i32,
                    score: e.score,
                    depth: e.depth as i32,
                });
            }
            let (br, bc) = b_move.map(|(r, c)| (r as i32, c as i32)).unwrap_or((-1, -1));
            unsafe {
                cb(
                    user_data,
                    c_entries.as_ptr(),
                    c_entries.len() as i32,
                    br,
                    bc,
                    b_score,
                );
            }
        }
    });

    let cb_ref: Option<&dyn Fn(&[TopMoveEntry], Option<(usize, usize)>, f32)> = rust_callback.as_ref().map(|cb| cb as &dyn Fn(&[TopMoveEntry], Option<(usize, usize)>, f32));

    let (best_move, score, stats) = engine.search_single_depth_with_callback(&board, p, depth as u8, cancel_flag, live_nodes, cb_ref);

    unsafe {
        if let Some((r, c)) = best_move {
            if !out_r.is_null() {
                *out_r = r as i32;
            }
            if !out_c.is_null() {
                *out_c = c as i32;
            }
        } else {
            if !out_r.is_null() {
                *out_r = -1;
            }
            if !out_c.is_null() {
                *out_c = -1;
            }
        }

        if !out_score.is_null() {
            *out_score = score;
        }
        if !out_nodes.is_null() {
            *out_nodes = stats.nodes;
        }

        if !out_top_moves.is_null() && !out_num_top.is_null() && max_top > 0 {
            let count = stats.top_moves.len().min(max_top as usize);
            let out_slice = std::slice::from_raw_parts_mut(out_top_moves, count);
            for i in 0..count {
                let tm = &stats.top_moves[i];
                out_slice[i] = CTopMoveEntry {
                    rank: tm.rank as i32,
                    r: tm.r as i32,
                    c: tm.c as i32,
                    score: tm.score,
                    depth: tm.depth as i32,
                };
            }
            *out_num_top = count as i32;
        }
    }

    1
}

/// Usage:
///     hex_engine_get_strategy(board_flat, size, player, out_intent_buf, buf_len, out_threat);
/// Usage Example:
///     hex_engine_get_strategy(board, 11, 2, buf, 128, &threat);
/// Description:
///     Returns strategic guidance, macro game plan, and threat level for the current board state.
#[no_mangle]
pub extern "C" fn hex_engine_get_strategy(
    board_flat: *const u8,
    size: i32,
    player: i32,
    out_intent_buf: *mut std::os::raw::c_char,
    buf_len: i32,
    out_threat: *mut i32,
) -> i32 {
    if board_flat.is_null() || size <= 0 || size > 14 {
        return 0;
    }

    let s = size as usize;
    let slice = unsafe { std::slice::from_raw_parts(board_flat, s * s) };
    let mut board = HexBoard::new(s);

    for r in 0..s {
        for c in 0..s {
            let p = slice[r * s + c];
            if p == RED || p == BLUE {
                board.place_move(r, c, p);
            }
        }
    }

    let guide = patterns::HexPatternMatcher::get_strategic_guidance(&board, player as u8);

    unsafe {
        if !out_threat.is_null() {
            *out_threat = guide.threat_level as i32;
        }
        if !out_intent_buf.is_null() && buf_len > 0 {
            let intent_bytes = guide.intent.as_bytes();
            let copy_len = intent_bytes.len().min((buf_len - 1) as usize);
            std::ptr::copy_nonoverlapping(intent_bytes.as_ptr(), out_intent_buf as *mut u8, copy_len);
            *out_intent_buf.add(copy_len) = 0;
        }
    }

    1
}

/// Usage:
///     hex_engine_clear_tt(engine_ptr);
/// Usage Example:
///     hex_engine_clear_tt(engine);
/// Description:
///     Clears transposition table entries in SearchEngine.
#[no_mangle]
pub extern "C" fn hex_engine_clear_tt(engine_ptr: *mut SearchEngine) {
    if !engine_ptr.is_null() {
        let engine = unsafe { &*engine_ptr };
        engine.tt.clear();
    }
}

/// Usage:
///     hex_engine_get_book_moves(board_flat, size, player, out_r, out_c, max_moves, out_count);
/// Usage Example:
///     hex_engine_get_book_moves(grid, 11, 2, r_buf, c_buf, 16, &count);
/// Description:
///     Queries all valid game-theoretic opening book moves for the current board configuration.
#[no_mangle]
pub extern "C" fn hex_engine_get_book_moves(
    board_flat: *const u8,
    size: i32,
    player: i32,
    out_r: *mut i32,
    out_c: *mut i32,
    max_moves: i32,
    out_count: *mut i32,
) -> i32 {
    if board_flat.is_null() || size <= 0 || out_count.is_null() {
        return 0;
    }

    let mut board = HexBoard::new(size as usize);
    let s = size as usize;
    let slice = unsafe { std::slice::from_raw_parts(board_flat, s * s) };
    for r in 0..s {
        for c in 0..s {
            let p = slice[r * s + c];
            if p == RED || p == BLUE {
                board.place_move(r, c, p);
            }
        }
    }

    let book_moves = openings::HexOpeningBook::get_all_book_moves(&board, player as u8);
    let count = book_moves.len().min(max_moves as usize);

    unsafe {
        if !out_r.is_null() && !out_c.is_null() {
            let r_slice = std::slice::from_raw_parts_mut(out_r, count);
            let c_slice = std::slice::from_raw_parts_mut(out_c, count);
            for (i, &(r, c)) in book_moves.iter().take(count).enumerate() {
                r_slice[i] = r as i32;
                c_slice[i] = c as i32;
            }
        }
        *out_count = count as i32;
    }

    1
}

/// Usage:
///     hex_engine_get_initial_candidates(engine, board_flat, size, player, out_top, 12, &num_top);
/// Usage Example:
///     hex_engine_get_initial_candidates(engine, grid, 11, 2, top_buf, 12, &num);
/// Description:
///     Returns instant candidate move leaderboard for the board state using TT lookup and fast heuristic.
#[no_mangle]
pub extern "C" fn hex_engine_get_initial_candidates(
    engine_ptr: *mut SearchEngine,
    board_flat: *const u8,
    size: i32,
    player: i32,
    out_top_moves: *mut CTopMoveEntry,
    max_top: i32,
    out_num_top: *mut i32,
) -> i32 {
    if engine_ptr.is_null() || board_flat.is_null() || size <= 0 || size > 14 || out_top_moves.is_null() || out_num_top.is_null() || max_top <= 0 {
        return 0;
    }

    let engine = unsafe { &*engine_ptr };
    let s = size as usize;
    let slice = unsafe { std::slice::from_raw_parts(board_flat, s * s) };
    let mut board = HexBoard::new(s);

    for r in 0..s {
        for c in 0..s {
            let p = slice[r * s + c];
            if p == RED || p == BLUE {
                board.place_move(r, c, p);
            }
        }
    }

    let p = player as u8;
    board.set_current_player(p);
    let top_moves = engine.get_initial_candidates(&board, p, max_top as usize);
    let count = top_moves.len().min(max_top as usize);
    let out_slice = unsafe { std::slice::from_raw_parts_mut(out_top_moves, count) };

    for i in 0..count {
        let tm = &top_moves[i];
        out_slice[i] = CTopMoveEntry {
            rank: tm.rank as i32,
            r: tm.r as i32,
            c: tm.c as i32,
            score: tm.score,
            depth: tm.depth as i32,
        };
    }
    unsafe {
        *out_num_top = count as i32;
    }

    1
}

/// Usage:
///     let nic = nic_engine_create(3);
/// Usage Example:
///     let nic = nic_engine_create(3);
/// Description:
///     Creates and heap-allocates a new NicReplicaEngine instance.
#[no_mangle]
pub extern "C" fn nic_engine_create(depth: i32) -> *mut nic_replica::NicReplicaEngine {
    Box::into_raw(Box::new(nic_replica::NicReplicaEngine::with_depth(depth.max(1) as usize)))
}

/// Usage:
///     nic_engine_destroy(nic_ptr);
/// Usage Example:
///     nic_engine_destroy(nic);
/// Description:
///     Frees a heap-allocated NicReplicaEngine instance.
#[no_mangle]
pub extern "C" fn nic_engine_destroy(nic: *mut nic_replica::NicReplicaEngine) {
    if !nic.is_null() {
        unsafe {
            drop(Box::from_raw(nic));
        }
    }
}

/// Usage:
///     let success = nic_engine_select_move(nic, board_flat, 11, BLUE, &mut out_r, &mut out_c, &mut out_score);
/// Usage Example:
///     let res = nic_engine_select_move(nic, board_ptr, 11, 2, &mut r, &mut c, &mut score);
/// Description:
///     Executes move selection using the NIC replica engine algorithm.
#[no_mangle]
pub extern "C" fn nic_engine_select_move(
    nic_ptr: *mut nic_replica::NicReplicaEngine,
    board_flat: *const u8,
    size: i32,
    player: i32,
    out_r: *mut i32,
    out_c: *mut i32,
    out_score: *mut f32,
) -> i32 {
    if nic_ptr.is_null() || board_flat.is_null() || size <= 0 || size > 14 || out_r.is_null() || out_c.is_null() || out_score.is_null() {
        return 0;
    }

    let nic = unsafe { &mut *nic_ptr };
    let s = size as usize;
    let slice = unsafe { std::slice::from_raw_parts(board_flat, s * s) };
    let mut board = HexBoard::new(s);

    for r in 0..s {
        for c in 0..s {
            let p = slice[r * s + c];
            if p == RED || p == BLUE {
                board.place_move(r, c, p);
            }
        }
    }

    let (bm, score) = nic.select_move(&board, player as u8);
    if let Some((r, c)) = bm {
        unsafe {
            *out_r = r as i32;
            *out_c = c as i32;
            *out_score = score;
        }
        1
    } else {
        0
    }
}

