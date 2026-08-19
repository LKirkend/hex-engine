//! ============================================================================
//! Nintendo Impossible Computer (NIC) Replica Engine
//!
//! Description:
//!     A high-fidelity algorithmic replica of Nintendo's Impossible Computer (NIC)
//!     from Clubhouse Games: 51 Worldwide Classics.
//!
//! Architecture:
//!     1. Anshelevich H-Search Virtual Connection (VC) Hypergraph Deduction.
//!     2. Shannon Resistor Network Conductance Delta (ΔG) Gradient Evaluation.
//!     3. Precomputed Edge Templates (Edge-2, Edge-3, Edge-4).
//!     4. Greedy VC Wire Extension, Frontier Preemption, & Compulsory Carrier Defense.
//!
//! Author: Logan Kirkendall (Logan@LKAud.io)
//! License: MIT
//! ============================================================================

use crate::board::{HexBoard, BLUE, EMPTY, RED};
use crate::patterns::HexPatternMatcher;
use crate::resistance::ResistanceEvaluator;

/// Represents a candidate move evaluated by the NIC replica.
#[derive(Debug, Clone, Copy)]
pub struct NicMoveScore {
    pub r: usize,
    pub c: usize,
    pub score: f32,
    pub delta_g_own: f32,
    pub delta_g_opp: f32,
    pub vc_bonus: f32,
}

/// Nintendo Impossible Computer (NIC) algorithmic replica engine.
///
/// Implements pure Anshelevich Virtual Connection deduction, electrical conductance
/// gradient search, edge template rails, and greedy VC tree growth.
pub struct NicReplicaEngine {
    pub max_depth: usize,
}

impl NicReplicaEngine {
    /// Usage:
    ///     let nic = NicReplicaEngine::new();
    /// Usage Example:
    ///     let nic = NicReplicaEngine::new();
    /// Description:
    ///     Initializes a new NIC replica engine instance with default shallow VC search.
    pub fn new() -> Self {
        Self { max_depth: 3 }
    }

    /// Usage:
    ///     let nic = NicReplicaEngine::with_depth(depth);
    /// Usage Example:
    ///     let nic = NicReplicaEngine::with_depth(4);
    /// Description:
    ///     Initializes a new NIC replica engine instance with a specific search depth.
    pub fn with_depth(depth: usize) -> Self {
        Self { max_depth: depth }
    }

    /// Usage:
    ///     let (best_move, score) = nic.select_move(board, player);
    /// Usage Example:
    ///     let mut nic = NicReplicaEngine::new();
    ///     let (bm, score) = nic.select_move(&board, RED);
    /// Description:
    ///     Selects the optimal move according to Nintendo's Anshelevich VC deduction
    ///     and Shannon conductance gradient algorithm.
    pub fn select_move(&mut self, board: &HexBoard, player: u8) -> (Option<(usize, usize)>, f32) {
        if board.is_game_over() {
            return (None, 0.0);
        }

        // 1. Compulsory Carrier Response ($O(1)$ immediate tactical reply)
        if let Some(compulsory) = HexPatternMatcher::get_compulsory_carrier_response(board, player) {
            return (Some(compulsory), 1000.0);
        }

        // 2. Immediate Winning Move Detection (0-cost virtual wire completion)
        let legal_moves = board.get_legal_moves();
        if legal_moves.is_empty() {
            return (None, 0.0);
        }
        if legal_moves.len() == 1 {
            return (Some(legal_moves[0]), 0.0);
        }

        let opponent = if player == RED { BLUE } else { RED };

        // 3. Evaluate Conductance Gradient & VC Bonuses for all Legal Moves
        let mut scored_moves: Vec<NicMoveScore> = Vec::with_capacity(legal_moves.len());

        let base_r_own = ResistanceEvaluator::compute_player_resistance(board, player).max(0.001);
        let base_r_opp = ResistanceEvaluator::compute_player_resistance(board, opponent).max(0.001);
        let base_g_own = 1.0 / base_r_own;
        let base_g_opp = 1.0 / base_r_opp;

        for &(r, c) in &legal_moves {
            let mut clone = board.clone();
            clone.place_move(r, c, player);

            // Immediate win check
            if clone.get_winner() == player {
                return (Some((r, c)), 10000.0);
            }

            // Resistance change after move
            let after_r_own = ResistanceEvaluator::compute_player_resistance(&clone, player).max(0.001);
            let after_r_opp = ResistanceEvaluator::compute_player_resistance(&clone, opponent).max(0.001);
            let after_g_own = 1.0 / after_r_own;
            let after_g_opp = 1.0 / after_r_opp;

            let delta_g_own = (after_g_own - base_g_own).max(0.0);
            let delta_g_opp = (base_g_opp - after_g_opp).max(0.0);

            // Anshelevich VC bonuses
            let vc_bonus = HexPatternMatcher::evaluate_pattern_bonus(board, r, c, player);

            // Distance improvement
            let my_dist = crate::evaluator::HexEvaluator::shortest_path(board, player);
            let my_dist_after = crate::evaluator::HexEvaluator::shortest_path(&clone, player);
            let dist_bonus = if my_dist_after < my_dist {
                (my_dist - my_dist_after) as f32 * 60.0
            } else {
                0.0
            };

            // Opponent shortest-path disruption
            let opp_dist = crate::evaluator::HexEvaluator::shortest_path(board, opponent);
            let opp_dist_after = crate::evaluator::HexEvaluator::shortest_path(&clone, opponent);
            let opp_dist_penalty = if opp_dist_after > opp_dist {
                (opp_dist_after - opp_dist) as f32 * 75.0
            } else {
                0.0
            };

            // Composite NIC scoring formula:
            // Score = 120 * ΔG_own + 90 * ΔG_opp + VC_bonus + Path_deltas
            let total_score = delta_g_own * 120.0
                + delta_g_opp * 90.0
                + vc_bonus
                + dist_bonus
                + opp_dist_penalty;

            scored_moves.push(NicMoveScore {
                r,
                c,
                score: total_score,
                delta_g_own,
                delta_g_opp,
                vc_bonus,
            });
        }

        // Sort descending by score
        scored_moves.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // 4. Shallow Minimax Verification over Top-5 Candidate Moves
        if self.max_depth <= 1 || scored_moves.len() <= 1 {
            let best = scored_moves[0];
            return (Some((best.r, best.c)), best.score);
        }

        let mut best_move = (scored_moves[0].r, scored_moves[0].c);
        let mut best_eval = -100000.0f32;

        let candidate_count = 6.min(scored_moves.len());
        for i in 0..candidate_count {
            let cand = scored_moves[i];
            let mut clone = board.clone();
            clone.place_move(cand.r, cand.c, player);

            let eval = -self.shallow_search(&clone, opponent, self.max_depth - 1, -100000.0, 100000.0);
            if eval > best_eval {
                best_eval = eval;
                best_move = (cand.r, cand.c);
            }
        }

        (Some(best_move), best_eval)
    }

    /// Usage:
    ///     let eval = nic.shallow_search(board, player, depth, alpha, beta);
    /// Description:
    ///     Performs a shallow minimax tree search using pure Anshelevich VC evaluation.
    fn shallow_search(&mut self, board: &HexBoard, player: u8, depth: usize, mut alpha: f32, beta: f32) -> f32 {
        let winner = board.get_winner();
        if winner == player {
            return 10000.0;
        } else if winner != EMPTY {
            return -10000.0;
        }

        if depth == 0 {
            return self.evaluate_leaf(board, player);
        }

        let legal_moves = board.get_legal_moves();
        if legal_moves.is_empty() {
            return 0.0;
        }

        let opponent = if player == RED { BLUE } else { RED };

        // Quick compulsory reply
        if let Some((cr, cc)) = HexPatternMatcher::get_compulsory_carrier_response(board, player) {
            let mut clone = board.clone();
            clone.place_move(cr, cc, player);
            return -self.shallow_search(&clone, opponent, depth - 1, -beta, -alpha);
        }

        // Rank top moves for shallow expansion
        let mut moves: Vec<((usize, usize), f32)> = Vec::with_capacity(legal_moves.len());
        for &(r, c) in &legal_moves {
            let pat = HexPatternMatcher::evaluate_pattern_bonus(board, r, c, player);
            moves.push(((r, c), pat));
        }
        moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let explore_count = 4.min(moves.len());
        for i in 0..explore_count {
            let ((r, c), _) = moves[i];
            let mut clone = board.clone();
            clone.place_move(r, c, player);

            let score = -self.shallow_search(&clone, opponent, depth - 1, -beta, -alpha);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// Usage:
    ///     let score = nic.evaluate_leaf(board, player);
    /// Description:
    ///     Leaf evaluation combining conductance and shortest path distances.
    fn evaluate_leaf(&self, board: &HexBoard, player: u8) -> f32 {
        let opponent = if player == RED { BLUE } else { RED };
        let r_own = ResistanceEvaluator::compute_player_resistance(board, player).max(0.001);
        let r_opp = ResistanceEvaluator::compute_player_resistance(board, opponent).max(0.001);
        let g_own = 1.0 / r_own;
        let g_opp = 1.0 / r_opp;

        let d_own = crate::evaluator::HexEvaluator::shortest_path(board, player) as f32;
        let d_opp = crate::evaluator::HexEvaluator::shortest_path(board, opponent) as f32;

        (g_own - g_opp) * 80.0 + (d_opp - d_own) * 45.0
    }
}
