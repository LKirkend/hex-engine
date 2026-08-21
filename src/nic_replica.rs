// ============================================================================
// Nintendo Impossible Computer (NIC) Replica Engine
//
// Description:
//     A faithful algorithmic replica of Nintendo's Impossible Computer (NIC)
//     from Clubhouse Games: 51 Worldwide Classics.
//
// Architecture:
//     Pure 1-ply greedy conductance-maximizer. No minimax tree search.
//     The real NIC is a casual-game AI that plays the move maximizing its own
//     conductance improvement while minimizing the opponent's, evaluated
//     instantaneously without lookahead.
//
//     1. Compulsory carrier defense (forced tactical reply).
//     2. Immediate winning move detection.
//     3. Must-block opponent's immediate win.
//     4. For each legal move: compute delta_G_own and delta_G_opp via resistance network.
//     5. Score = delta_G_own * 100 + delta_G_opp * 80 + small center bias.
//     6. Play the highest-scoring move.
//
// Author: Logan Kirkendall (Logan@LKAud.io)
// License: MIT
// ============================================================================

use crate::board::{HexBoard, BLUE, RED};
use crate::patterns::HexPatternMatcher;
use crate::resistance::ResistanceEvaluator;

/// Represents a candidate move evaluated by the NIC replica.
///
/// Usage:
///     let ms = NicMoveScore { r: 5, c: 5, score: 150.0, delta_g_own: 0.8, delta_g_opp: 0.3 };
/// Description:
///     Captures the greedy conductance-delta scoring components for a single candidate move.
#[derive(Debug, Clone, Copy)]
pub struct NicMoveScore {
    pub r: usize,
    pub c: usize,
    pub score: f32,
    pub delta_g_own: f32,
    pub delta_g_opp: f32,
}

/// Nintendo Impossible Computer (NIC) algorithmic replica engine.
///
/// Implements pure 1-ply greedy conductance-maximization matching the real NIC's
/// behavior: no minimax, no tree search, just the move with the best immediate
/// resistance-network improvement. Compulsory carrier defense is the only
/// "lookahead" — it's pattern-matched, not searched.
///
/// Usage:
///     let mut nic = NicReplicaEngine::new();
///     let (best_move, score) = nic.select_move(&board, RED);
pub struct NicReplicaEngine;

impl NicReplicaEngine {
    /// Usage:
    ///     let nic = NicReplicaEngine::new();
    /// Usage Example:
    ///     let nic = NicReplicaEngine::new();
    /// Description:
    ///     Initializes a new NIC replica engine instance. No parameters needed
    ///     since the NIC uses pure greedy evaluation with no depth parameter.
    pub fn new() -> Self {
        Self
    }

    /// Usage:
    ///     let nic = NicReplicaEngine::with_depth(_depth);
    /// Usage Example:
    ///     let nic = NicReplicaEngine::with_depth(3); // depth ignored
    /// Description:
    ///     Backward-compatible constructor. Depth parameter is ignored since the
    ///     real NIC is purely greedy (no tree search).
    pub fn with_depth(_depth: usize) -> Self {
        Self
    }

    /// Usage:
    ///     let (best_move, score) = nic.select_move(board, player);
    /// Usage Example:
    ///     let mut nic = NicReplicaEngine::new();
    ///     let (bm, score) = nic.select_move(&board, RED);
    /// Description:
    ///     Selects the optimal move using pure greedy conductance-delta evaluation.
    ///     Matches the real NIC's decision process:
    ///     1. Compulsory carrier defense (instant forced reply)
    ///     2. Immediate win detection
    ///     3. Must-block opponent win
    ///     4. Greedy ΔG maximization for all remaining legal moves
    pub fn select_move(&mut self, board: &HexBoard, player: u8) -> (Option<(usize, usize)>, f32) {
        if board.is_game_over() {
            return (None, 0.0);
        }

        // 1. Compulsory Carrier Response (instant pattern-matched tactical reply)
        if let Some(compulsory) = HexPatternMatcher::get_compulsory_carrier_response(board, player) {
            return (Some(compulsory), 1000.0);
        }

        let legal_moves = board.get_legal_moves();
        if legal_moves.is_empty() {
            return (None, 0.0);
        }
        if legal_moves.len() == 1 {
            return (Some(legal_moves[0]), 0.0);
        }

        let opponent = if player == RED { BLUE } else { RED };

        // 2. Immediate Winning Move Detection
        for &(r, c) in &legal_moves {
            let mut clone = board.clone();
            clone.place_move(r, c, player);
            if clone.get_winner() == player {
                return (Some((r, c)), 10000.0);
            }
        }

        // 3. Must-Block Opponent's Immediate Win
        for &(r, c) in &legal_moves {
            let mut clone = board.clone();
            clone.place_move(r, c, opponent);
            if clone.get_winner() == opponent {
                return (Some((r, c)), 9000.0);
            }
        }

        // 4. Pure Greedy Conductance-Delta Evaluation for All Legal Moves
        let base_r_own = ResistanceEvaluator::compute_player_resistance(board, player).max(0.001);
        let base_r_opp = ResistanceEvaluator::compute_player_resistance(board, opponent).max(0.001);
        let base_g_own = 1.0 / base_r_own;
        let base_g_opp = 1.0 / base_r_opp;

        let size = board.size;
        let center = (size - 1) as f32 / 2.0;

        let mut scored_moves: Vec<NicMoveScore> = Vec::with_capacity(legal_moves.len());

        for &(r, c) in &legal_moves {
            let mut clone = board.clone();
            clone.place_move(r, c, player);

            // Conductance after placing our stone
            let after_r_own = ResistanceEvaluator::compute_player_resistance(&clone, player).max(0.001);
            let after_r_opp = ResistanceEvaluator::compute_player_resistance(&clone, opponent).max(0.001);
            let after_g_own = 1.0 / after_r_own;
            let after_g_opp = 1.0 / after_r_opp;

            // ΔG: how much our conductance improved, how much opponent's dropped
            let delta_g_own = (after_g_own - base_g_own).max(0.0);
            let delta_g_opp = (base_g_opp - after_g_opp).max(0.0);

            // Small center bias (NIC slightly prefers central moves, all else equal)
            let dr = r as f32 - center;
            let dc = c as f32 - center;
            let center_bias = (center - (dr * dr + dc * dc).sqrt()) * 0.5;

            // NIC composite score: pure conductance deltas + tiny center tiebreaker
            let total_score = delta_g_own * 100.0
                + delta_g_opp * 80.0
                + center_bias;

            scored_moves.push(NicMoveScore {
                r,
                c,
                score: total_score,
                delta_g_own,
                delta_g_opp,
            });
        }

        // Sort descending by score, play the best
        scored_moves.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let best = scored_moves[0];
        (Some((best.r, best.c)), best.score)
    }
}
