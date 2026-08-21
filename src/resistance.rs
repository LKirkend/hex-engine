//! Resistance-Based Network Evaluator for Hex.
//!
//! OOP Description:
//! The `ResistanceEvaluator` struct models the Hex board as an electrical resistance
//! network. Own stones have zero resistance (wires), empty cells have unit resistance,
//! and opponent stones have infinite resistance (open circuits). The effective resistance
//! from source edge to sink edge is computed via Gauss-Seidel iterative relaxation.
//! The ratio R_opponent / R_self provides a highly accurate, continuous evaluation
//! that naturally accounts for virtual connections, dead cells, and path multiplicity.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use crate::board::{HexBoard, BLUE, EMPTY, RED};

const DR: [isize; 6] = [-1, -1, 0, 0, 1, 1];
const DC: [isize; 6] = [0, 1, -1, 1, -1, 0];

/// Maximum board dimension supported by the resistance solver.
const MAX_CELLS: usize = 196; // 14x14

pub struct ResistanceEvaluator;

impl ResistanceEvaluator {
    /// Usage:
    ///     let (r_red, r_blue) = ResistanceEvaluator::compute_resistances(board);
    /// Usage Example:
    ///     let (r_red, r_blue) = ResistanceEvaluator::compute_resistances(&board);
    ///     let advantage = r_blue - r_red; // positive = Red advantage
    /// Description:
    ///     Computes the effective electrical resistance for both Red (top-to-bottom)
    ///     and Blue (left-to-right) using Gauss-Seidel iterative relaxation.
    ///     Lower resistance = stronger connection = better position.
    ///     Returns (R_red, R_blue) where smaller is better for that player.
    pub fn compute_resistances(board: &HexBoard) -> (f32, f32) {
        let r_red = Self::compute_player_resistance(board, RED);
        let r_blue = Self::compute_player_resistance(board, BLUE);
        (r_red, r_blue)
    }

    /// Usage:
    ///     let eval = ResistanceEvaluator::evaluate_for_player(board, player);
    /// Usage Example:
    ///     let score = ResistanceEvaluator::evaluate_for_player(&board, RED);
    /// Description:
    ///     Returns a heuristic score from the perspective of `player`.
    ///     Positive means `player` has advantage, negative means opponent has advantage.
    ///     Score is scaled to be comparable with the existing evaluator's range (~±100).
    pub fn evaluate_for_player(board: &HexBoard, player: u8) -> f32 {
        let (r_red, r_blue) = Self::compute_resistances(board);

        // Avoid division by zero: if resistance is 0, the player has already won
        if player == RED {
            if r_red <= 0.001 { return 500.0; }
            if r_blue <= 0.001 { return -500.0; }
            // Log-ratio: positive when Red has lower resistance (advantage)
            (r_blue.ln() - r_red.ln()) * 30.0
        } else {
            if r_blue <= 0.001 { return 500.0; }
            if r_red <= 0.001 { return -500.0; }
            (r_red.ln() - r_blue.ln()) * 30.0
        }
    }

    /// Usage:
    ///     let resistance = ResistanceEvaluator::compute_player_resistance(board, RED);
    /// Usage Example:
    ///     let r_red = ResistanceEvaluator::compute_player_resistance(&board, RED);
    /// Description:
    ///     Computes effective resistance for a single player's connection goal.
    ///     Red connects top-to-bottom (rows 0 to N-1), Blue connects left-to-right (cols 0 to N-1).
    ///     Uses Gauss-Seidel iterative relaxation to solve the Kirchhoff equations.
    ///     Own stones = superconductor (0 resistance), empty = unit resistor, opponent = insulator.
    ///     2-bridges and precomputed edge templates provide direct high-conductance virtual links.
    pub fn compute_player_resistance(board: &HexBoard, player: u8) -> f32 {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };

        // Node voltages: source edge = 1.0, sink edge = 0.0
        let mut voltage = [0.0f32; MAX_CELLS];

        // Initialize: linear interpolation along connection axis
        for r in 0..size {
            for c in 0..size {
                let idx = r * size + c;
                let cell = board.get_cell(r, c);
                if cell == opponent {
                    voltage[idx] = 0.0;
                    continue;
                }
                if player == RED {
                    voltage[idx] = 1.0 - (r as f32 / (size - 1) as f32);
                } else {
                    voltage[idx] = 1.0 - (c as f32 / (size - 1) as f32);
                }
            }
        }

        // Precompute 2-bridge virtual connections and edge template connections
        let mut virtual_links = [[0usize; 6]; MAX_CELLS];
        let mut virtual_link_count = [0u8; MAX_CELLS];
        let mut carrier_conductance = [0.0f32; MAX_CELLS];
        let mut source_template_g = [0.0f32; MAX_CELLS];
        let mut sink_template_g = [0.0f32; MAX_CELLS];

        Self::compute_virtual_network(
            board,
            player,
            size,
            &mut virtual_links,
            &mut virtual_link_count,
            &mut carrier_conductance,
            &mut source_template_g,
            &mut sink_template_g,
        );

        // Gauss-Seidel iteration (16 iterations: fast and accurate convergence)
        let iterations = 16;
        for _iter in 0..iterations {
            for r in 0..size {
                for c in 0..size {
                    let idx = r * size + c;
                    let cell = board.get_cell(r, c);
                    if cell == opponent { continue; }

                    let is_source = if player == RED { r == 0 } else { c == 0 };
                    let is_sink = if player == RED { r == size - 1 } else { c == size - 1 };

                    if is_source && cell == player {
                        voltage[idx] = 1.0;
                        continue;
                    }
                    if is_sink && cell == player {
                        voltage[idx] = 0.0;
                        continue;
                    }

                    let (mut sum_gv, mut sum_g) = Self::neighbor_conductance_sum(
                        board, player, size, r, c, &voltage,
                    );

                    // Add direct 2-bridge virtual links to friendly partner stones
                    let link_cnt = virtual_link_count[idx] as usize;
                    for k in 0..link_cnt {
                        let target_idx = virtual_links[idx][k];
                        let g_bridge = 50.0f32;
                        sum_gv += g_bridge * voltage[target_idx];
                        sum_g += g_bridge;
                    }

                    // Add carrier conductance for empty bridge cells
                    if carrier_conductance[idx] > 0.0 {
                        sum_g += carrier_conductance[idx];
                    }

                    // Add rail connection for boundary cells or edge template virtual connections
                    let (rail_gv, rail_g) = if is_source {
                        (1.0, 1.0)
                    } else if is_sink {
                        (0.0, 1.0)
                    } else {
                        (0.0, 0.0)
                    };

                    sum_gv += rail_gv + source_template_g[idx] * 1.0 + sink_template_g[idx] * 0.0;
                    sum_g += rail_g + source_template_g[idx] + sink_template_g[idx];

                    if sum_g > 0.0 {
                        voltage[idx] = sum_gv / sum_g;
                    }
                }
            }
        }

        // Compute total current from source rail (boundary + edge template direct current)
        let mut total_current = 0.0f32;
        for r in 0..size {
            for c in 0..size {
                let idx = r * size + c;
                let cell = board.get_cell(r, c);
                if cell == opponent { continue; }

                let is_source = if player == RED { r == 0 } else { c == 0 };

                if is_source {
                    if cell == player {
                        // Current from source stone to non-source neighbors
                        for k in 0..6 {
                            let nr = r as isize + DR[k];
                            let nc = c as isize + DC[k];
                            if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                                let nidx = nr as usize * size + nc as usize;
                                let n_cell = board.get_cell(nr as usize, nc as usize);
                                if n_cell == opponent { continue; }
                                let g = if n_cell == player { 100.0 } else { 2.0 };
                                let delta = voltage[idx] - voltage[nidx];
                                if delta > 0.0 {
                                    total_current += g * delta;
                                }
                            }
                        }
                    } else {
                        // Empty source-edge cell: current from source rail
                        total_current += 1.0 * (1.0 - voltage[idx]);
                    }
                } else if source_template_g[idx] > 0.0 {
                    // Current flowing from source rail through precomputed edge template into inner stone
                    total_current += source_template_g[idx] * (1.0 - voltage[idx]);
                }
            }
        }

        if total_current > 0.001 {
            1.0 / total_current
        } else {
            1000.0
        }
    }

    /// Usage:
    ///     let (sum_gv, sum_g) = Self::neighbor_conductance_sum(...);
    /// Description:
    ///     Computes weighted sum of neighbor voltages and total conductance for Gauss-Seidel.
    #[inline(always)]
    fn neighbor_conductance_sum(
        board: &HexBoard,
        player: u8,
        size: usize,
        r: usize,
        c: usize,
        voltage: &[f32; MAX_CELLS],
    ) -> (f32, f32) {
        let opponent = if player == RED { BLUE } else { RED };
        let cell = board.get_cell(r, c);
        let mut sum_gv = 0.0f32;
        let mut sum_g = 0.0f32;

        for k in 0..6 {
            let nr = r as isize + DR[k];
            let nc = c as isize + DC[k];
            if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                let nidx = nr as usize * size + nc as usize;
                let n_cell = board.get_cell(nr as usize, nc as usize);
                if n_cell == opponent { continue; }

                let g = match (cell, n_cell) {
                    (p, n) if p == player && n == player => 100.0,
                    (p, EMPTY) if p == player => 2.0,
                    (EMPTY, n) if n == player => 2.0,
                    (EMPTY, EMPTY) => 1.0,
                    _ => 1.0,
                };

                sum_gv += g * voltage[nidx];
                sum_g += g;
            }
        }

        (sum_gv, sum_g)
    }

    /// Usage:
    ///     Self::compute_virtual_network(board, player, size, &mut links, &mut counts, &mut carrier_g, &mut src_g, &mut snk_g);
    /// Description:
    ///     Precomputes 2-bridge virtual connections and edge-template rail couplings.
    fn compute_virtual_network(
        board: &HexBoard,
        player: u8,
        size: usize,
        virtual_links: &mut [[usize; 6]; MAX_CELLS],
        virtual_link_count: &mut [u8; MAX_CELLS],
        carrier_g: &mut [f32; MAX_CELLS],
        source_template_g: &mut [f32; MAX_CELLS],
        sink_template_g: &mut [f32; MAX_CELLS],
    ) {
        const B2: [(isize, isize, isize, isize, isize, isize); 6] = [
            (-2, 1, -1, 0, -1, 1),
            (-1, -1, -1, 0, 0, -1),
            (-1, 2, -1, 1, 0, 1),
            (1, -2, 0, -1, 1, -1),
            (1, 1, 0, 1, 1, 0),
            (2, -1, 1, -1, 1, 0),
        ];

        for r in 0..size {
            for c in 0..size {
                let idx = r * size + c;
                if board.get_cell(r, c) != player { continue; }
                let ur = r as isize;
                let uc = c as isize;

                // 1. Internal stone-to-stone 2-bridges
                for k in 0..6 {
                    let (br, bc, c1r, c1c, c2r, c2c) = B2[k];
                    let nr = ur + br;
                    let nc = uc + bc;
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        if board.get_cell(nr as usize, nc as usize) != player { continue; }
                        let cr1 = ur + c1r;
                        let cc1 = uc + c1c;
                        let cr2 = ur + c2r;
                        let cc2 = uc + c2c;
                        if cr1 >= 0 && cr1 < size as isize && cc1 >= 0 && cc1 < size as isize
                            && cr2 >= 0 && cr2 < size as isize && cc2 >= 0 && cc2 < size as isize
                        {
                            if board.get_cell(cr1 as usize, cc1 as usize) == EMPTY
                                && board.get_cell(cr2 as usize, cc2 as usize) == EMPTY
                            {
                                let nidx = nr as usize * size + nc as usize;
                                let cnt = virtual_link_count[idx] as usize;
                                if cnt < 6 {
                                    virtual_links[idx][cnt] = nidx;
                                    virtual_link_count[idx] += 1;
                                }

                                let idx1 = cr1 as usize * size + cc1 as usize;
                                let idx2 = cr2 as usize * size + cc2 as usize;
                                carrier_g[idx1] += 15.0;
                                carrier_g[idx2] += 15.0;
                            }
                        }
                    }
                }

                // 2. Precomputed edge template connections to Source and Sink rails
                if crate::patterns::HexPatternMatcher::is_stone_connected_to_source_edge(board, r, c, player) {
                    let d = if player == RED { r } else { c };
                    source_template_g[idx] = if d == 1 { 35.0 } else { 18.0 };
                }
                if crate::patterns::HexPatternMatcher::is_stone_connected_to_sink_edge(board, r, c, player) {
                    let d = if player == RED { size - 1 - r } else { size - 1 - c };
                    sink_template_g[idx] = if d == 1 { 35.0 } else { 18.0 };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::HexBoard;

    #[test]
    fn test_empty_board_symmetric() {
        let board = HexBoard::new(5);
        let (r_red, r_blue) = ResistanceEvaluator::compute_resistances(&board);
        let ratio = r_red / r_blue;
        assert!(ratio > 0.5 && ratio < 2.0, "Empty board resistance ratio should be near 1.0, got {}", ratio);
    }

    #[test]
    fn test_center_stone_advantage() {
        let mut board = HexBoard::new(5);
        board.place_move(2, 2, RED);
        let (r_red, r_blue) = ResistanceEvaluator::compute_resistances(&board);
        assert!(r_red < r_blue, "Center stone should give Red lower resistance: R_red={}, R_blue={}", r_red, r_blue);
    }

    #[test]
    fn test_winning_path_zero_resistance() {
        let mut board = HexBoard::new(3);
        board.place_move(0, 1, RED);
        board.place_move(1, 1, RED);
        board.place_move(2, 1, RED);
        let (r_red, _) = ResistanceEvaluator::compute_resistances(&board);
        assert!(r_red < 0.1, "Winning path should have near-zero resistance, got {}", r_red);
    }

    #[test]
    fn test_blocked_path_high_resistance() {
        let mut board = HexBoard::new(5);
        for c in 0..5 {
            board.place_move(2, c, BLUE);
        }
        let (r_red, _) = ResistanceEvaluator::compute_resistances(&board);
        assert!(r_red > 5.0, "Blocked path should have high resistance, got {}", r_red);
    }

    #[test]
    fn test_evaluate_for_player_direction() {
        let mut board = HexBoard::new(7);
        board.place_move(3, 3, RED);
        let red_eval = ResistanceEvaluator::evaluate_for_player(&board, RED);
        let blue_eval = ResistanceEvaluator::evaluate_for_player(&board, BLUE);
        assert!(red_eval > 0.0, "Red should have positive eval with center stone: {}", red_eval);
        assert!(blue_eval < 0.0, "Blue should have negative eval when Red has center: {}", blue_eval);
    }
}
