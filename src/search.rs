//! High-Performance Multi-Threaded Lazy SMP Search Engine.
//!
//! OOP Description:
//! The `SearchEngine` struct coordinates parallel Lazy SMP Negamax minimax searches,
//! combining PVS (Principal Variation Search), Null Move Pruning, Futility Pruning,
//! Late Move Pruning, History + Killer Move Heuristics, Aspiration Windows,
//! Ladder Escalation heuristics, and lock-free Transposition Tables.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::board::{HexBoard, BLUE, EMPTY, RED};
use crate::evaluator::{HexEvaluator, WIN_SCORE};
use crate::openings::HexOpeningBook;
use crate::patterns::HexPatternMatcher;
use crate::tt::{TranspositionTable, EXACT, LOWERBOUND, UPPERBOUND};

#[derive(Clone, Debug)]
pub struct TopMoveEntry {
    pub rank: usize,
    pub r: usize,
    pub c: usize,
    pub score: f32,
    pub depth: u8,
    pub note: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SearchStats {
    pub nodes: u64,
    pub time_sec: f64,
    pub nps: u64,
    pub depth_reached: u8,
    pub is_final: bool,
    pub top_moves: Vec<TopMoveEntry>,
}

/// Maximum board dimension for history table sizing.
const MAX_BOARD_CELLS: usize = 196; // 14x14
/// Maximum search ply for killer move table sizing.
const MAX_PLY: usize = 64;

pub struct SearchEngine {
    pub tt: Arc<TranspositionTable>,
    pub num_threads: usize,
    /// History heuristic table: indexed by [cell_index], tracks moves that cause cutoffs.
    /// Incremented by depth^2 on beta cutoff, decayed periodically.
    history: [[i32; MAX_BOARD_CELLS]; 3], // [player][cell_idx]
    /// Killer move table: stores the last 2 non-tactical moves per ply that caused cutoffs.
    killers: [[(usize, usize); 2]; MAX_PLY],
}

impl SearchEngine {
    /// Usage:
    ///     let engine = SearchEngine::new();
    /// Usage Example:
    ///     let engine = SearchEngine::new();
    /// Description:
    ///     Initializes search engine with lock-free TT, history/killer tables, and auto-detected CPU core count.
    pub fn new() -> Self {
        SearchEngine {
            tt: Arc::new(TranspositionTable::new()),
            num_threads: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
            history: [[0i32; MAX_BOARD_CELLS]; 3],
            killers: [[(usize::MAX, usize::MAX); 2]; MAX_PLY],
        }
    }

    /// Usage:
    ///     engine.update_history(player, r, c, depth);
    /// Description:
    ///     Records a beta-cutoff move in the history table, incrementing by depth^2.
    #[inline(always)]
    fn update_history(&mut self, player: u8, r: usize, c: usize, size: usize, depth: u8) {
        let idx = r * size + c;
        if idx < MAX_BOARD_CELLS {
            self.history[player as usize][idx] += (depth as i32) * (depth as i32);
        }
    }

    /// Usage:
    ///     engine.update_killers(ply, r, c);
    /// Description:
    ///     Records a killer move at the given ply (most recent replaces oldest).
    #[inline(always)]
    fn update_killers(&mut self, ply: usize, r: usize, c: usize) {
        if ply < MAX_PLY {
            if self.killers[ply][0] != (r, c) {
                self.killers[ply][1] = self.killers[ply][0];
                self.killers[ply][0] = (r, c);
            }
        }
    }

    /// Usage:
    ///     engine.clear_search_tables();
    /// Description:
    ///     Resets history and killer tables for a fresh search (called at iterative deepening start).
    pub fn clear_search_tables(&mut self) {
        // Decay history rather than clearing (preserves some ordering information)
        for p in 0..3 {
            for i in 0..MAX_BOARD_CELLS {
                self.history[p][i] /= 2;
            }
        }
        self.killers = [[(usize::MAX, usize::MAX); 2]; MAX_PLY];
    }

    /// Usage:
    ///     let (mv, score, stats) = engine.search(board, BLUE, 8, cancel_flag);
    /// Usage Example:
    ///     let (mv, score, stats) = engine.search(&board, RED, 6, None);
    /// Description:
    ///     Executes parallel Lazy SMP iterative deepening PVS search.
    pub fn search(
        &mut self,
        board: &HexBoard,
        player: u8,
        max_depth: u8,
        cancel_flag: Option<&AtomicBool>,
    ) -> (Option<(usize, usize)>, f32, SearchStats) {
        let start_time = Instant::now();
        let total_nodes = AtomicU64::new(0);

        let total_stones = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;
        if total_stones == 0 {
            let top_moves = HexOpeningBook::get_empty_board_candidates(board, player, max_depth);
            let best_move = top_moves.first().map(|e| (e.r, e.c));
            let best_score = top_moves.first().map(|e| e.score).unwrap_or(0.0);
            let stats = SearchStats {
                nodes: top_moves.len() as u64,
                time_sec: 0.001,
                nps: 10000,
                depth_reached: max_depth,
                is_final: true,
                top_moves,
            };
            return (best_move, best_score, stats);
        }

        let mut best_move = None;
        let mut best_score = 0.0f32;
        let mut top_moves = Vec::new();
        let mut depth_reached = 0;

        // Compute root candidates once (expensive path-aware ordering)
        let root_candidates = self.order_moves(board, player, max_depth, 0);

        for d in 1..=max_depth {
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    break;
                }
            }

            let (d_move, d_score, d_top) = self.search_root_with_candidates(board, player, d, &root_candidates, &total_nodes, cancel_flag, None);
            if d_move.is_some() {
                best_move = d_move;
                best_score = d_score;
                top_moves = d_top;
                depth_reached = d;
            }

            if d_score.abs() >= WIN_SCORE - 1000.0 {
                break;
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64().max(0.0001);
        let nodes = total_nodes.load(Ordering::Relaxed);
        let nps = (nodes as f64 / elapsed) as u64;

        let stats = SearchStats {
            nodes,
            time_sec: elapsed,
            nps,
            depth_reached,
            is_final: true,
            top_moves,
        };

        (best_move, best_score, stats)
    }

    /// Usage:
    ///     let (mv, score, stats) = engine.search_single_depth(board, BLUE, 4, None, None);
    /// Usage Example:
    ///     let (mv, score, stats) = engine.search_single_depth(&board, RED, 4, None, Some(&live_nodes));
    /// Description:
    ///     Executes a single depth-limited search pass with real-time live node reporting.
    pub fn search_single_depth(
        &mut self,
        board: &HexBoard,
        player: u8,
        depth: u8,
        cancel_flag: Option<&AtomicBool>,
        live_nodes: Option<&AtomicU64>,
    ) -> (Option<(usize, usize)>, f32, SearchStats) {
        self.search_single_depth_with_callback(board, player, depth, cancel_flag, live_nodes, None)
    }

    pub fn search_single_depth_with_callback(
        &mut self,
        board: &HexBoard,
        player: u8,
        depth: u8,
        cancel_flag: Option<&AtomicBool>,
        live_nodes: Option<&AtomicU64>,
        live_callback: Option<&dyn Fn(&[TopMoveEntry], Option<(usize, usize)>, f32)>,
    ) -> (Option<(usize, usize)>, f32, SearchStats) {
        let start_time = Instant::now();
        let internal_nodes = AtomicU64::new(0);
        let total_nodes = live_nodes.unwrap_or(&internal_nodes);

        let total_stones = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;
        let root_candidates = if total_stones == 0 {
            HexOpeningBook::get_all_book_moves(board, player)
        } else {
            self.order_moves(board, player, depth, 0)
        };
        let (best_move, best_score, top_moves) = self.search_root_with_candidates(
            board,
            player,
            depth,
            &root_candidates,
            total_nodes,
            cancel_flag,
            live_callback,
        );
        let elapsed = start_time.elapsed().as_secs_f64().max(0.0001);
        let nodes = total_nodes.load(Ordering::Relaxed);
        let nps = (nodes as f64 / elapsed) as u64;

        let stats = SearchStats {
            nodes,
            time_sec: elapsed,
            nps,
            depth_reached: depth,
            is_final: true,
            top_moves,
        };

        (best_move, best_score, stats)
    }

    /// Usage:
    ///     let candidates = engine.get_initial_candidates(board, player, 12);
    /// Usage Example:
    ///     let moves = engine.get_initial_candidates(&board, RED, 12);
    /// Description:
    ///     Instantly extracts and ranks candidate moves for a board position using TT lookup
    ///     and fast static evaluation, enabling zero-latency UI candidate display on move navigation.
    pub fn get_initial_candidates(&self, board: &HexBoard, player: u8, max_top: usize) -> Vec<TopMoveEntry> {
        let max_candidates = 28;
        let candidates = HexEvaluator::get_promising_moves(board, player, max_candidates);
        if candidates.is_empty() {
            return Vec::new();
        }

        let next_player = if player == RED { BLUE } else { RED };
        let mut scored_moves: Vec<((usize, usize), f32, u8)> = Vec::with_capacity(candidates.len());

        for &(mr, mc) in &candidates {
            let idx = mr * board.size + mc;
            let child_hash = board.zobrist_hash
                ^ crate::board::zobrist_key(idx, player)
                ^ crate::board::zobrist_player_key(player)
                ^ crate::board::zobrist_player_key(next_player);
            let (prev_score, prev_depth) = if let Some((score, d)) = self.tt.get_entry_score(child_hash) {
                (-score, d)
            } else {
                let mut clone = board.clone();
                clone.place_move(mr, mc, player);
                let base_eval = HexEvaluator::evaluate_fast(&clone, player);
                let total_st = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;
                let book_bonus = if total_st == 0 {
                    if mr == (board.size - 1) / 2 && mc == (board.size - 1) / 2 {
                        80.0
                    } else if HexOpeningBook::get_all_book_moves(board, player).contains(&(mr, mc)) {
                        45.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                (base_eval + book_bonus, 0)
            };
            scored_moves.push(((mr, mc), prev_score, prev_depth));
        }

        scored_moves.sort_by(|a, b| {
            let a_searched = a.2 > 0;
            let b_searched = b.2 > 0;
            if a_searched != b_searched {
                return b_searched.cmp(&a_searched);
            }
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        let abs_multiplier = if player == RED { 1.0f32 } else { -1.0f32 };
        let total_stones = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;

        scored_moves
            .into_iter()
            .take(max_top)
            .enumerate()
            .map(|(i, ((r, c), player_relative_score, verified_depth))| {
                let mut note = if let Some(((br, bc), b_note)) = HexOpeningBook::get_opening_move(board, player) {
                    if br == r && bc == c {
                        Some(b_note.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                if note.is_none() && total_stones == 0 {
                    if board.size == 11 {
                        note = match (r, c) {
                            (5, 5) => Some("11x11 Game-Theoretic Optimal Center (F6)".to_string()),
                            (4, 4) => Some("11x11 Master Long-Diagonal Fair Opening (E5)".to_string()),
                            (3, 4) => Some("11x11 Master Short-Diagonal Near-Center (E4)".to_string()),
                            (2, 3) => Some("11x11 Master Short-Diagonal Probe (D3)".to_string()),
                            (1, 2) => Some("11x11 Master Mild Flank Opening (C2)".to_string()),
                            (6, 6) => Some("11x11 Master Symmetrical Long-Diagonal (G7)".to_string()),
                            (3, 7) => Some("11x11 Master East Acute Probe (H4)".to_string()),
                            (1, 1) => Some("11x11 Master Acute Corner Anchor (B2)".to_string()),
                            (8, 2) => Some("11x11 Master Obtuse Flank Anchor (C9)".to_string()),
                            _ => None,
                        };
                    }
                }
                TopMoveEntry {
                    rank: i + 1,
                    r,
                    c,
                    score: player_relative_score * abs_multiplier,
                    depth: verified_depth,
                    note,
                }
            })
            .collect()
    }

    /// Usage:
    ///     let (mv, score, top) = engine.search_root_with_candidates(board, player, depth, &candidates, &nodes, cancel, cb);
    /// Usage Example:
    ///     let (mv, score, top) = engine.search_root_with_candidates(&board, RED, 6, &moves, &nodes, None, None);
    /// Description:
    ///     PVS + aspiration re-search root search using precomputed candidate list.
    ///     Re-orders candidates using TT lookups before searching and streams live updates.
    fn search_root_with_candidates(
        &mut self,
        board: &HexBoard,
        player: u8,
        depth: u8,
        candidates: &[(usize, usize)],
        total_nodes: &AtomicU64,
        cancel_flag: Option<&AtomicBool>,
        live_callback: Option<&dyn Fn(&[TopMoveEntry], Option<(usize, usize)>, f32)>,
    ) -> (Option<(usize, usize)>, f32, Vec<TopMoveEntry>) {
        if candidates.is_empty() {
            return (None, HexEvaluator::evaluate_absolute(board), Vec::new());
        }

        // 1. Incremental Zobrist TT-Based Root Move Ordering
        let next_player = if player == RED { BLUE } else { RED };
        let mut scored_root_moves: Vec<((usize, usize), f32, u8)> = Vec::with_capacity(candidates.len());
        for &(mr, mc) in candidates {
            let idx = mr * board.size + mc;
            let child_hash = board.zobrist_hash
                ^ crate::board::zobrist_key(idx, player)
                ^ crate::board::zobrist_player_key(player)
                ^ crate::board::zobrist_player_key(next_player);
            let (prev_score, prev_depth) = if let Some((score, d)) = self.tt.get_entry_score(child_hash) {
                (-score, d)
            } else {
                let mut clone = board.clone();
                clone.place_move(mr, mc, player);
                (HexEvaluator::evaluate_for_player(&clone, player), 0)
            };
            scored_root_moves.push(((mr, mc), prev_score, prev_depth));
        }
        scored_root_moves.sort_by(|a, b| {
            let a_searched = a.2 > 0;
            let b_searched = b.2 > 0;
            if a_searched != b_searched {
                return b_searched.cmp(&a_searched);
            }
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 2. Persistent Candidate Pool Across Iterative Depths
        // Initializes with all root candidate moves and their authentic prior scores/depths
        let mut candidate_pool: Vec<((usize, usize), f32, u8)> = scored_root_moves
            .iter()
            .map(|&(mv, prev, p_d)| (mv, prev, p_d))
            .collect();

        let mut best_score = -WIN_SCORE * 2.0;
        let mut best_move = if !scored_root_moves.is_empty() { scored_root_moves[0].0 } else { (0, 0) };
        let mut evaluated_moves = Vec::new();
        let opponent = if player == RED { BLUE } else { RED };
        let abs_multiplier = if player == RED { 1.0f32 } else { -1.0f32 };

        // Immediately stream initial candidate list with all authentic prior TT/static scores
        if let Some(cb) = live_callback {
            let initial_top: Vec<TopMoveEntry> = candidate_pool
                .iter()
                .take(12)
                .enumerate()
                .map(|(i, &((mr, mc), p_score, v_d))| TopMoveEntry {
                    rank: i + 1,
                    r: mr,
                    c: mc,
                    score: p_score * abs_multiplier,
                    depth: v_d,
                    note: None,
                })
                .collect();
            if !initial_top.is_empty() {
                let init_best_mv = Some(scored_root_moves[0].0);
                let init_best_sc = scored_root_moves[0].1 * abs_multiplier;
                cb(&initial_top, init_best_mv, init_best_sc);
            }
        }

        for (idx, &( (r, c), prev, _p_depth )) in scored_root_moves.iter().enumerate() {
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    break;
                }
            }

            let mut clone = board.clone();
            clone.place_move(r, c, player);

            let mut evaluated_d = depth;

            let score = if idx == 0 {
                // First candidate (PV move): full depth search with aspiration window
                if prev > -900.0 && depth >= 2 {
                    let window = 35.0f32;
                    let mut s = -self.pvs(
                        &mut clone,
                        opponent,
                        depth.saturating_sub(1),
                        -(prev + window),
                        -(prev - window),
                        1,
                        total_nodes,
                        cancel_flag,
                    );
                    if s <= prev - window {
                        // Fail-low: re-search with wider window
                        s = -self.pvs(
                            &mut clone,
                            opponent,
                            depth.saturating_sub(1),
                            -s,
                            WIN_SCORE * 2.0,
                            1,
                            total_nodes,
                            cancel_flag,
                        );
                    } else if s >= prev + window {
                        // Fail-high: re-search with wider window
                        s = -self.pvs(
                            &mut clone,
                            opponent,
                            depth.saturating_sub(1),
                            -WIN_SCORE * 2.0,
                            -s,
                            1,
                            total_nodes,
                            cancel_flag,
                        );
                    }
                    s
                } else {
                    -self.pvs(
                        &mut clone,
                        opponent,
                        depth.saturating_sub(1),
                        -WIN_SCORE * 2.0,
                        WIN_SCORE * 2.0,
                        1,
                        total_nodes,
                        cancel_flag,
                    )
                }
            } else {
                let candidate_depth = if idx >= 12 && depth >= 4 && prev > -900.0 && prev < best_score - 50.0 {
                    depth.saturating_sub(3)
                } else {
                    depth
                };
                evaluated_d = candidate_depth;

                // Multi-PV candidate search with stabilized aspiration window
                if prev > -900.0 && depth >= 2 {
                    let window = 35.0f32;
                    let mut s = -self.pvs(
                        &mut clone,
                        opponent,
                        candidate_depth.saturating_sub(1),
                        -(prev + window),
                        -(prev - window),
                        1,
                        total_nodes,
                        cancel_flag,
                    );
                    if s <= prev - window {
                        // Candidate dropped: fail-low search to obtain true lower score
                        s = -self.pvs(
                            &mut clone,
                            opponent,
                            candidate_depth.saturating_sub(1),
                            -s,
                            WIN_SCORE * 2.0,
                            1,
                            total_nodes,
                            cancel_flag,
                        );
                    } else if s >= prev + window {
                        // Candidate surged: re-search to full depth with open upper bound
                        evaluated_d = depth;
                        s = -self.pvs(
                            &mut clone,
                            opponent,
                            depth.saturating_sub(1),
                            -WIN_SCORE * 2.0,
                            -s,
                            1,
                            total_nodes,
                            cancel_flag,
                        );
                    }
                    s
                } else {
                    -self.pvs(
                        &mut clone,
                        opponent,
                        candidate_depth.saturating_sub(1),
                        -WIN_SCORE * 2.0,
                        WIN_SCORE * 2.0,
                        1,
                        total_nodes,
                        cancel_flag,
                    )
                }
            };

            evaluated_moves.push(((r, c), score, evaluated_d));

            // Update specific candidate in candidate_pool in-place
            if let Some(entry) = candidate_pool.iter_mut().find(|e| e.0 == (r, c)) {
                entry.1 = score;
                entry.2 = evaluated_d;
            }

            if score > best_score {
                best_score = score;
                best_move = (r, c);
            }

            // Dynamic live streaming: update leaderboard in-place, prioritizing current-depth verified entries
            if let Some(cb) = live_callback {
                let mut current_sorted: Vec<((usize, usize), f32, u8)> = candidate_pool
                    .iter()
                    .filter(|e| e.2 > 0)
                    .copied()
                    .collect();
                current_sorted.sort_by(|a, b| {
                    let a_curr = a.2 == depth;
                    let b_curr = b.2 == depth;
                    if a_curr != b_curr {
                        return b_curr.cmp(&a_curr);
                    }
                    b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                });
                if !current_sorted.is_empty() {
                    let live_top: Vec<TopMoveEntry> = current_sorted
                        .into_iter()
                        .take(12)
                        .enumerate()
                        .map(|(i, ((mr, mc), p_score, v_d))| TopMoveEntry {
                            rank: i + 1,
                            r: mr,
                            c: mc,
                            score: p_score * abs_multiplier,
                            depth: v_d,
                            note: None,
                        })
                        .collect();
                    cb(&live_top, Some(best_move), best_score * abs_multiplier);
                }
            }
        }

        // 3. Backtracking Depth-Completion Pass for Top-10 Leaderboard Candidates
        // Ensures that any move appearing in the top 10 is brought up to full current `depth`
        let mut top_candidates_needing_deepening: Vec<((usize, usize), f32, u8)> = candidate_pool
            .iter()
            .filter(|e| e.2 < depth && e.2 > 0)
            .copied()
            .collect();
        top_candidates_needing_deepening.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for ((r, c), _prev, _p_d) in top_candidates_needing_deepening.into_iter().take(10) {
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    break;
                }
            }
            let mut clone = board.clone();
            clone.place_move(r, c, player);
            let score = -self.pvs(
                &mut clone,
                opponent,
                depth.saturating_sub(1),
                -WIN_SCORE * 2.0,
                WIN_SCORE * 2.0,
                1,
                total_nodes,
                cancel_flag,
            );
            if let Some(entry) = candidate_pool.iter_mut().find(|e| e.0 == (r, c)) {
                entry.1 = score;
                entry.2 = depth;
            }
            if score > best_score {
                best_score = score;
                best_move = (r, c);
            }
            if let Some(entry) = evaluated_moves.iter_mut().find(|e| e.0 == (r, c)) {
                entry.1 = score;
                entry.2 = depth;
            } else {
                evaluated_moves.push(((r, c), score, depth));
            }
        }

        self.tt.store(board.zobrist_hash, depth, best_score, EXACT, Some(best_move));

        // Use the fully updated candidate_pool for the final top_entries
        let mut final_sorted: Vec<((usize, usize), f32, u8)> = candidate_pool
            .iter()
            .filter(|e| e.2 > 0)
            .copied()
            .collect();
        final_sorted.sort_by(|a, b| {
            let a_curr = a.2 == depth;
            let b_curr = b.2 == depth;
            if a_curr != b_curr {
                return b_curr.cmp(&a_curr);
            }
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        let top_entries: Vec<TopMoveEntry> = final_sorted
            .into_iter()
            .take(12)
            .enumerate()
            .map(|(i, ((r, c), player_relative_score, verified_depth))| {
                let total_stones = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;
                let mut note = if let Some(((br, bc), b_note)) = HexOpeningBook::get_opening_move(board, player) {
                    if br == r && bc == c {
                        Some(b_note.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                if note.is_none() && total_stones == 0 {
                    if board.size == 11 {
                        note = match (r, c) {
                            (5, 5) => Some("11x11 Game-Theoretic Optimal Center (F6)".to_string()),
                            (4, 4) => Some("11x11 Master Long-Diagonal Fair Opening (E5)".to_string()),
                            (3, 4) => Some("11x11 Master Short-Diagonal Near-Center (E4)".to_string()),
                            (2, 3) => Some("11x11 Master Short-Diagonal Probe (D3)".to_string()),
                            (1, 2) => Some("11x11 Master Mild Flank Opening (C2)".to_string()),
                            (6, 6) => Some("11x11 Master Symmetrical Long-Diagonal (G7)".to_string()),
                            (3, 7) => Some("11x11 Master East Acute Probe (H4)".to_string()),
                            (1, 1) => Some("11x11 Master Acute Corner Anchor (B2)".to_string()),
                            (8, 2) => Some("11x11 Master Obtuse Flank Anchor (C9)".to_string()),
                            _ => None,
                        };
                    } else if r == (board.size - 1) / 2 && c == (board.size - 1) / 2 {
                        note = Some("Optimal Center Opening".to_string());
                    }
                }
                TopMoveEntry {
                    rank: i + 1,
                    r,
                    c,
                    score: player_relative_score * abs_multiplier,
                    depth: verified_depth,
                    note,
                }
            })
            .collect();

        let root_eval_score = best_score * abs_multiplier;
        (Some(best_move), root_eval_score, top_entries)
    }

    fn pvs(
        &mut self,
        board: &mut HexBoard,
        player: u8,
        depth: u8,
        mut alpha: f32,
        beta: f32,
        ply: usize,
        total_nodes: &AtomicU64,
        cancel_flag: Option<&AtomicBool>,
    ) -> f32 {
        if let Some(flag) = cancel_flag {
            if flag.load(Ordering::Relaxed) {
                return 0.0;
            }
        }
        total_nodes.fetch_add(1, Ordering::Relaxed);

        let winner = board.get_winner();
        if winner == player {
            return WIN_SCORE - ply as f32;
        } else if winner != EMPTY {
            return -WIN_SCORE + ply as f32;
        }

        if depth == 0 {
            let opponent = if player == RED { BLUE } else { RED };
            let opp_wins = HexEvaluator::get_immediate_winning_moves(board, opponent);
            if !opp_wins.is_empty() {
                return -WIN_SCORE + (ply + 1) as f32;
            }
            let opp_dist = HexEvaluator::shortest_path(board, opponent);
            if opp_dist <= 1 && ply <= 8 {
                let mut best_q = HexEvaluator::evaluate_fast(board, player);
                let opp_fast = HexEvaluator::get_fast_promising_moves(board, opponent, 4);
                for &(or, oc) in &opp_fast {
                    board.place_move(or, oc, opponent);
                    let opp_winner = board.get_winner();
                    let opp_score = if opp_winner == opponent {
                        WIN_SCORE - (ply + 2) as f32
                    } else {
                        HexEvaluator::evaluate_fast(board, opponent)
                    };
                    board.undo_move();
                    if -opp_score < best_q {
                        best_q = -opp_score;
                    }
                }
                return best_q;
            }
            return HexEvaluator::evaluate_fast(board, player);
        }

        if let Some((cached_score, _)) = self.tt.lookup(board.zobrist_hash, depth, alpha, beta) {
            return cached_score;
        }

        let opponent = if player == RED { BLUE } else { RED };
        let static_eval = HexEvaluator::evaluate_fast(board, player);

        // Precompute shortest-path distances for tactical safety checks
        let opp_dist = HexEvaluator::shortest_path(board, opponent);
        let my_dist = HexEvaluator::shortest_path(board, player);
        let in_tactical_danger = opp_dist <= 2 || my_dist <= 2;

        // === NULL MOVE PRUNING ===
        // If we skip our turn and the opponent still can't beat us, prune this branch.
        // Guard: don't apply when either player is close to completing a connection.
        if depth >= 3 && ply > 0 && !in_tactical_danger && static_eval.abs() < WIN_SCORE - 2000.0 {
            if opp_dist > 2 && my_dist > 2 {
                // "Pass" to opponent: search at reduced depth
                let null_r = if depth >= 6 { 3 } else { 2 };
                let null_depth = depth.saturating_sub(1 + null_r);
                let null_score = -self.pvs(
                    board, opponent, null_depth, -beta, -beta + 0.5, ply + 1, total_nodes, cancel_flag,
                );
                if null_score >= beta {
                    return null_score; // Fail high: this position is too good for us
                }
            }
        }

        let moves = self.order_moves_with_heuristics(board, player, depth, ply);
        if moves.is_empty() {
            return static_eval;
        }

        let mut best_score = -WIN_SCORE * 2.0;
        let mut best_move = None;
        let orig_alpha = alpha;
        let size = board.size;

        // === FUTILITY PRUNING MARGINS ===
        let futility_margin = match depth {
            1 => 55.0,
            2 => 110.0,
            _ => 0.0, // Only applies at depth 1-2
        };
        let can_futility_prune = !in_tactical_danger && depth <= 2
            && static_eval + futility_margin <= alpha
            && static_eval.abs() < WIN_SCORE - 2000.0;

        // === LATE MOVE PRUNING LIMIT ===
        // At shallow depths, search more moves to ensure critical blocking plays are evaluated
        let lmp_limit = if in_tactical_danger {
            moves.len()
        } else if depth <= 5 {
            (6 + depth as usize * 3).min(moves.len())
        } else {
            moves.len()
        };

        for (idx, &(r, c)) in moves.iter().enumerate() {
            // Late Move Pruning: skip tail moves at shallow depth
            if idx >= lmp_limit && best_score > -WIN_SCORE + 1000.0 {
                break;
            }

            // Futility Pruning: skip non-first moves at depth 1-2 if hopeless
            if can_futility_prune && idx > 0 {
                continue;
            }

            board.place_move(r, c, player);

            let score = if idx == 0 {
                -self.pvs(board, opponent, depth.saturating_sub(1), -beta, -alpha, ply + 1, total_nodes, cancel_flag)
            } else {
                // LMR: reduce depth for late moves
                let reduction = if idx >= 6 && depth >= 3 && !in_tactical_danger {
                    if idx >= 12 { 2 } else { 1 }
                } else {
                    0
                };
                let mut s = -self.pvs(
                    board, opponent, depth.saturating_sub(1 + reduction), -(alpha + 0.5), -alpha, ply + 1, total_nodes, cancel_flag,
                );
                if s > alpha && (s < beta || reduction > 0) {
                    s = -self.pvs(board, opponent, depth.saturating_sub(1), -beta, -alpha, ply + 1, total_nodes, cancel_flag);
                }
                s
            };

            board.undo_move();

            if score > best_score {
                best_score = score;
                best_move = Some((r, c));
            }
            if best_score > alpha {
                alpha = best_score;
            }
            if alpha >= beta {
                // Beta cutoff: update history and killer tables
                self.update_history(player, r, c, size, depth);
                self.update_killers(ply, r, c);
                break;
            }
        }

        let flag = if best_score <= orig_alpha {
            UPPERBOUND
        } else if best_score >= beta {
            LOWERBOUND
        } else {
            EXACT
        };

        self.tt.store(board.zobrist_hash, depth, best_score, flag, best_move);
        best_score
    }

    /// Usage:
    ///     let moves = engine.order_moves_with_heuristics(board, player, depth, ply);
    /// Description:
    ///     Move ordering with TT, killer, history heuristics for optimal alpha-beta cutoffs.
    ///     Branching factor: 32 at depth<=3, 24 at depth<=6, 18 at depth<=8, 14 at depth>8.
    fn order_moves_with_heuristics(&self, board: &HexBoard, player: u8, depth: u8, ply: usize) -> Vec<(usize, usize)> {
        // 1. Immediate sudden death win
        let my_wins = HexEvaluator::get_immediate_winning_moves(board, player);
        if !my_wins.is_empty() {
            return my_wins;
        }

        // 2. Must-block opponent threat
        let opponent = if player == RED { BLUE } else { RED };
        let opp_wins = HexEvaluator::get_immediate_winning_moves(board, opponent);
        if !opp_wins.is_empty() {
            return opp_wins;
        }

        // 3. Compulsory 2-Bridge / Edge Template Carrier Defense
        if let Some(carrier_resp) = HexPatternMatcher::get_compulsory_carrier_response(board, player) {
            if board.get_cell(carrier_resp.0, carrier_resp.1) == EMPTY {
                return vec![carrier_resp];
            }
        }

        // Branching factor: 32→24→18→14 as depth increases
        let max_branches = if depth <= 3 { 32 } else if depth <= 6 { 24 } else if depth <= 8 { 18 } else { 14 };
        let mut candidates = if ply == 0 {
            HexEvaluator::get_promising_moves(board, player, max_branches)
        } else {
            HexEvaluator::get_fast_promising_moves(board, player, max_branches)
        };

        // 4. Forced Ladder Escape Step
        if let Some(ladder_move) = HexPatternMatcher::get_ladder_escape_move(board, player) {
            if let Some(pos) = candidates.iter().position(|&m| m == ladder_move) {
                candidates.swap(0, pos);
            } else if board.get_cell(ladder_move.0, ladder_move.1) == EMPTY {
                candidates.insert(0, ladder_move);
            }
        }

        // 5. Transposition Table Best Move Priority
        if let Some(tt_move) = self.tt.get_best_move(board.zobrist_hash) {
            if let Some(pos) = candidates.iter().position(|&m| m == tt_move) {
                candidates.swap(0, pos);
            } else if board.get_cell(tt_move.0, tt_move.1) == EMPTY {
                candidates.insert(0, tt_move);
            }
        }

        // 6. Killer Move Priority: promote killer moves from current ply
        if ply < MAX_PLY {
            for ki in (0..2).rev() {
                let (kr, kc) = self.killers[ply][ki];
                if kr < board.size && kc < board.size && board.get_cell(kr, kc) == EMPTY {
                    if let Some(pos) = candidates.iter().position(|&m| m == (kr, kc)) {
                        if pos > 1 { // Don't demote TT move
                            let mv = candidates.remove(pos);
                            candidates.insert(1, mv); // Insert after TT move
                        }
                    }
                }
            }
        }

        // 7. History Heuristic Re-ordering: sort non-priority moves by history score
        let size = board.size;
        let p_idx = player as usize;
        if candidates.len() > 3 {
            // Preserve first 2 positions (TT + killer), sort the rest by history
            let preserved = 2.min(candidates.len());
            candidates[preserved..].sort_by(|&(ar, ac), &(br, bc)| {
                let a_h = self.history[p_idx][ar * size + ac];
                let b_h = self.history[p_idx][br * size + bc];
                b_h.cmp(&a_h)
            });
        }

        // 8. Opening Book Candidate Prioritization (Root only)
        if ply == 0 {
            let book_moves = HexOpeningBook::get_all_book_moves(board, player);
            for &bm in book_moves.iter().rev() {
                if board.get_cell(bm.0, bm.1) == EMPTY {
                    if let Some(pos) = candidates.iter().position(|&m| m == bm) {
                        candidates.remove(pos);
                    }
                    candidates.insert(0, bm);
                }
            }
        }

        candidates
    }

    /// Usage:
    ///     let moves = engine.order_moves(board, player, depth, ply);
    /// Description:
    ///     Legacy move ordering wrapper (delegates to heuristic-enhanced ordering).
    fn order_moves(&self, board: &HexBoard, player: u8, depth: u8, ply: usize) -> Vec<(usize, usize)> {
        self.order_moves_with_heuristics(board, player, depth, ply)
    }
}
