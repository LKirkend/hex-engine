//! Hex Static Position Evaluator and Heuristic Module.
//!
//! OOP Description:
//! The `HexEvaluator` struct provides sub-microsecond static evaluation of Hex board
//! states using 2-bridge-aware 0-1 BFS shortest path distance, center dominance,
//! edge template virtual connections, tactical wedging, and SIMD sudden-death threat detection.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use crate::bitboard::Bitboard128;
use crate::board::{HexBoard, BLUE, EMPTY, RED};
use crate::patterns::HexPatternMatcher;
use crate::resistance::ResistanceEvaluator;

pub const WIN_SCORE: f32 = 100000.0;

pub const DR: [isize; 6] = [-1, -1, 0, 0, 1, 1];
pub const DC: [isize; 6] = [0, 1, -1, 1, -1, 0];

pub const B2_OFFSETS: [(isize, isize, isize, isize, isize, isize); 6] = [
    (-2, 1, -1, 0, -1, 1),
    (-1, -1, -1, 0, 0, -1),
    (-1, 2, -1, 1, 0, 1),
    (1, -2, 0, -1, 1, -1),
    (1, 1, 0, 1, 1, 0),
    (2, -1, 1, -1, 1, 0),
];

pub const CENTER_11: [f32; 121] = [
    -0.0000, 1.0019, 1.8602, 2.5289, 2.9581, 3.1066, 2.9581, 2.5289, 1.8602, 1.0019, -0.0000,
    1.0019, 2.1213, 3.1066, 3.8984, 4.4219, 4.6066, 4.4219, 3.8984, 3.1066, 2.1213, 1.0019,
    1.8602, 3.1066, 4.2426, 5.1983, 5.8632, 6.1066, 5.8632, 5.1983, 4.2426, 3.1066, 1.8602,
    2.5289, 3.8984, 5.1983, 6.3640, 7.2525, 7.6066, 7.2525, 6.3640, 5.1983, 3.8984, 2.5289,
    2.9581, 4.4219, 5.8632, 7.2525, 8.4853, 9.1066, 8.4853, 7.2525, 5.8632, 4.4219, 2.9581,
    3.1066, 4.6066, 6.1066, 7.6066, 9.1066, 10.6066, 9.1066, 7.6066, 6.1066, 4.6066, 3.1066,
    2.9581, 4.4219, 5.8632, 7.2525, 8.4853, 9.1066, 8.4853, 7.2525, 5.8632, 4.4219, 2.9581,
    2.5289, 3.8984, 5.1983, 6.3640, 7.2525, 7.6066, 7.2525, 6.3640, 5.1983, 3.8984, 2.5289,
    1.8602, 3.1066, 4.2426, 5.1983, 5.8632, 6.1066, 5.8632, 5.1983, 4.2426, 3.1066, 1.8602,
    1.0019, 2.1213, 3.1066, 3.8984, 4.4219, 4.6066, 4.4219, 3.8984, 3.1066, 2.1213, 1.0019,
    -0.0000, 1.0019, 1.8602, 2.5289, 2.9581, 3.1066, 2.9581, 2.5289, 1.8602, 1.0019, -0.0000
];

#[inline(always)]
fn get_center_weight(r: usize, c: usize, size: usize) -> f32 {
    let center = (size - 1) as f32 / 2.0;
    let max_dist = center * 1.414;
    let dr = r as f32 - center;
    let dc = c as f32 - center;
    let dist_c = (dr * dr + dc * dc).sqrt();
    (max_dist - dist_c) * 1.5
}

#[inline(always)]
fn evaluate_stone_bridges(board: &HexBoard, r: usize, c: usize, cell: u8, size: usize) -> f32 {
    let mut bridge_val = 0.0f32;
    let mut open_bridges = 0;
    let ur = r as isize;
    let uc = c as isize;

    for k in 0..6 {
        let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];
        let nr = ur + br;
        let nc = uc + bc;
        if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
            if board.get_cell(nr as usize, nc as usize) == cell {
                let c1_r = ur + c1r;
                let c1_c = uc + c1c;
                let c2_r = ur + c2r;
                let c2_c = uc + c2c;
                if c1_r >= 0 && c1_r < size as isize && c1_c >= 0 && c1_c < size as isize
                    && c2_r >= 0 && c2_r < size as isize && c2_c >= 0 && c2_c < size as isize
                {
                    let c1 = board.get_cell(c1_r as usize, c1_c as usize);
                    let c2 = board.get_cell(c2_r as usize, c2_c as usize);
                    if c1 == EMPTY && c2 == EMPTY {
                        open_bridges += 1;
                        bridge_val += if open_bridges == 1 { 14.0 } else { 18.0 };
                    }
                }
            }
        }
    }
    bridge_val
}

#[inline(always)]
fn evaluate_central_spine(board: &HexBoard, size: usize) -> f32 {
    if size < 7 { return 0.0; }
    let mut score = 0.0f32;
    let mid = (size - 1) / 2;

    let mut red_north_center = false;
    let mut red_center = false;
    let mut red_south_center = false;

    let mut blue_west_center = false;
    let mut blue_center = false;
    let mut blue_east_center = false;

    for r in (mid.saturating_sub(2))..=(mid + 2).min(size - 1) {
        for c in (mid.saturating_sub(2))..=(mid + 2).min(size - 1) {
            let cell = board.get_cell(r, c);
            if cell == RED {
                if r < mid { red_north_center = true; }
                else if r == mid { red_center = true; }
                else { red_south_center = true; }
            } else if cell == BLUE {
                if c < mid { blue_west_center = true; }
                else if c == mid { blue_center = true; }
                else { blue_east_center = true; }
            }
        }
    }

    if red_center && red_north_center && red_south_center {
        score += 32.0;
    } else if red_center && (red_north_center || red_south_center) {
        score += 14.0;
    }

    if blue_center && blue_west_center && blue_east_center {
        score -= 32.0;
    } else if blue_center && (blue_west_center || blue_east_center) {
        score -= 14.0;
    }

    score
}

pub struct HexEvaluator;

impl HexEvaluator {
    /// Usage:
    ///     let (reach_frac, chain_stones) = HexEvaluator::bridge_chain_reach(board, RED);
    /// Usage Example:
    ///     let (frac, cnt) = HexEvaluator::bridge_chain_reach(&board, RED);
    ///     if frac > 0.6 { /* Red has a dangerous bridge chain */ }
    /// Description:
    ///     Computes the maximum extent of a player's bridge-connected chain along their
    ///     connection axis. Uses BFS traversal where two friendly stones connected by a
    ///     2-bridge (both carrier cells empty) are treated as directly connected.
    ///     Returns (reach_fraction, chain_stone_count) where reach_fraction is the
    ///     fraction of the board axis spanned by the longest chain (0.0 to 1.0).
    pub fn bridge_chain_reach(board: &HexBoard, player: u8) -> (f32, usize) {
        let size = board.size;
        if size < 3 { return (0.0, 0); }
        let opponent = if player == RED { BLUE } else { RED };

        // Collect all stones of this player
        let bb = if player == RED { &board.red_bb } else { &board.blue_bb };
        let mut stones: Vec<usize> = Vec::new();
        let mut bits = bb.0;
        while bits != 0 {
            let idx = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            stones.push(idx);
        }
        if stones.is_empty() { return (0.0, 0); }

        // Build adjacency including 2-bridge virtual links
        let mut visited = [false; 196];
        let mut best_min_axis = size;
        let mut best_max_axis = 0usize;
        let mut best_chain_size = 0usize;

        for &start_idx in &stones {
            if visited[start_idx] { continue; }

            // BFS from this stone through direct neighbors and 2-bridges
            let mut queue = vec![start_idx];
            let mut component_min = size;
            let mut component_max = 0usize;
            let mut component_size = 0usize;
            visited[start_idx] = true;

            while let Some(cur) = queue.pop() {
                let cr = cur / size;
                let cc = cur % size;
                let axis_val = if player == RED { cr } else { cc };
                component_min = component_min.min(axis_val);
                component_max = component_max.max(axis_val);
                component_size += 1;

                // Direct hex neighbors
                for k in 0..6 {
                    let nr = cr as isize + DR[k];
                    let nc = cc as isize + DC[k];
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        let nidx = nr as usize * size + nc as usize;
                        if !visited[nidx] && board.get_cell(nr as usize, nc as usize) == player {
                            visited[nidx] = true;
                            queue.push(nidx);
                        }
                    }
                }

                // 2-bridge virtual connections
                let ur = cr as isize;
                let uc = cc as isize;
                for k in 0..6 {
                    let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];
                    let nr = ur + br;
                    let nc = uc + bc;
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        let nidx = nr as usize * size + nc as usize;
                        if !visited[nidx] && board.get_cell(nr as usize, nc as usize) == player {
                            // Check both carrier cells are empty
                            let cr1 = ur + c1r;
                            let cc1 = uc + c1c;
                            let cr2 = ur + c2r;
                            let cc2 = uc + c2c;
                            if cr1 >= 0 && cr1 < size as isize && cc1 >= 0 && cc1 < size as isize
                                && cr2 >= 0 && cr2 < size as isize && cc2 >= 0 && cc2 < size as isize
                            {
                                let c1 = board.get_cell(cr1 as usize, cc1 as usize);
                                let c2 = board.get_cell(cr2 as usize, cc2 as usize);
                                if c1 != opponent && c2 != opponent {
                                    // At least one carrier must be empty for the bridge to function
                                    if c1 == EMPTY || c2 == EMPTY {
                                        visited[nidx] = true;
                                        queue.push(nidx);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Also count edge connectivity: if component touches source/sink edges via
            // edge templates, extend the axis range
            // Source edge: row 0 for Red, col 0 for Blue
            // Sink edge: row size-1 for Red, col size-1 for Blue
            if player == RED {
                if component_min <= 2 {
                    // Check if any stone in the component connects to north edge via template
                    // Approximate: if min row is 0-2 and stone has template connection, treat as row 0
                    component_min = 0;
                }
                if component_max >= size - 3 {
                    component_max = size - 1;
                }
            } else {
                if component_min <= 2 {
                    component_min = 0;
                }
                if component_max >= size - 3 {
                    component_max = size - 1;
                }
            }

            let span = if component_max >= component_min { component_max - component_min } else { 0 };
            if span > best_max_axis - best_min_axis || (span == best_max_axis - best_min_axis && component_size > best_chain_size) {
                best_min_axis = component_min;
                best_max_axis = component_max;
                best_chain_size = component_size;
            }
        }

        let span = if best_max_axis >= best_min_axis { best_max_axis - best_min_axis } else { 0 };
        let reach_frac = span as f32 / (size - 1) as f32;
        (reach_frac, best_chain_size)
    }

    /// Usage:
    ///     let dist = HexEvaluator::shortest_path(board, RED);
    /// Usage Example:
    ///     let red_dist = HexEvaluator::shortest_path(&board, RED);
    /// Description:
    ///     Calculates the minimum stones required to complete a winning connection
    ///     using zero-allocation 0-1 BFS accounting for physical stones and established 2-bridges.
    #[inline(always)]
    pub fn shortest_path(board: &HexBoard, player: u8) -> i16 {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };

        let mut dist = [30000i16; 128];
        let mut deque = [0u8; 256];
        let mut head = 128usize;
        let mut tail = 128usize;

        if player == RED {
            for c in 0..size {
                let cell = board.get_cell(0, c);
                if cell == RED {
                    dist[c] = 0;
                    head -= 1;
                    deque[head] = c as u8;
                } else if cell == EMPTY {
                    dist[c] = 1;
                    deque[tail] = c as u8;
                    tail += 1;
                }
            }
            // Seed North Edge Templates (Edge-2 and Edge-3, rows 1 to 2)
            for r in 1..=2.min(size.saturating_sub(1)) {
                for c in 0..size {
                    if board.get_cell(r, c) == RED && crate::patterns::HexPatternMatcher::is_stone_connected_to_source_edge(board, r, c, RED) {
                        let idx = r * size + c;
                        if dist[idx] > 0 {
                            dist[idx] = 0;
                            head -= 1;
                            deque[head] = idx as u8;
                        }
                    }
                }
            }
        } else {
            for r in 0..size {
                let cell = board.get_cell(r, 0);
                let idx = r * size;
                if cell == BLUE {
                    dist[idx] = 0;
                    head -= 1;
                    deque[head] = idx as u8;
                } else if cell == EMPTY {
                    dist[idx] = 1;
                    deque[tail] = idx as u8;
                    tail += 1;
                }
            }
            // Seed West Edge Templates (Edge-2 and Edge-3, cols 1 to 2)
            for c in 1..=2.min(size.saturating_sub(1)) {
                for r in 0..size {
                    if board.get_cell(r, c) == BLUE && crate::patterns::HexPatternMatcher::is_stone_connected_to_source_edge(board, r, c, BLUE) {
                        let idx = r * size + c;
                        if dist[idx] > 0 {
                            dist[idx] = 0;
                            head -= 1;
                            deque[head] = idx as u8;
                        }
                    }
                }
            }
        }

        while head < tail {
            let u = deque[head] as usize;
            head += 1;
            let d = dist[u];
            let r = u / size;
            let c = u % size;

            if player == RED {
                if r == size - 1 {
                    return d;
                }
                if d == 0 && crate::patterns::HexPatternMatcher::is_stone_connected_to_sink_edge(board, r, c, RED) {
                    return 0;
                }
            }
            if player == BLUE {
                if c == size - 1 {
                    return d;
                }
                if d == 0 && crate::patterns::HexPatternMatcher::is_stone_connected_to_sink_edge(board, r, c, BLUE) {
                    return 0;
                }
            }

            // 1. Direct Neighbor Traversal
            for k in 0..6 {
                let nr = r as isize + DR[k];
                let nc = c as isize + DC[k];
                if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                    let v = nr as usize * size + nc as usize;
                    let cell = board.get_cell(nr as usize, nc as usize);
                    if cell != opponent {
                        let w = if cell == player { 0 } else { 1 };
                        if d + w < dist[v] {
                            dist[v] = d + w;
                            if w == 0 {
                                head -= 1;
                                deque[head] = v as u8;
                            } else {
                                deque[tail] = v as u8;
                                tail += 1;
                            }
                        }
                    }
                }
            }

            let u_cell = board.get_cell(r, c);
            if u_cell == player {
                let ur = r as isize;
                let uc = c as isize;
                for k in 0..6 {
                    let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];
                    let nr = ur + br;
                    let nc = uc + bc;
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        let cell = board.get_cell(nr as usize, nc as usize);
                        if cell == player {
                            let c1_r = ur + c1r;
                            let c1_c = uc + c1c;
                            let c2_r = ur + c2r;
                            let c2_c = uc + c2c;
                            if c1_r >= 0 && c1_r < size as isize && c1_c >= 0 && c1_c < size as isize
                                && c2_r >= 0 && c2_r < size as isize && c2_c >= 0 && c2_c < size as isize
                            {
                                let c1 = board.get_cell(c1_r as usize, c1_c as usize);
                                let c2 = board.get_cell(c2_r as usize, c2_c as usize);
                                if c1 == EMPTY && c2 == EMPTY {
                                    let v = nr as usize * size + nc as usize;
                                    if d < dist[v] {
                                        dist[v] = d;
                                        head -= 1;
                                        deque[head] = v as u8;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        30000
    }

    /// Usage:
    ///     let score = HexEvaluator::evaluate_for_player(board, RED);
    /// Usage Example:
    ///     let eval = HexEvaluator::evaluate_for_player(&board, current_player);
    /// Description:
    ///     Full-accuracy Negamax heuristic score blending resistance-based network
    ///     evaluation (60%) with heuristic features (40%). Use for root-level evaluation
    ///     and shallow candidate pre-scoring. For deep leaf nodes, use evaluate_fast().
    pub fn evaluate_for_player(board: &HexBoard, player: u8) -> f32 {
        let winner = board.get_winner();
        if winner == player {
            return WIN_SCORE;
        } else if winner != EMPTY {
            return -WIN_SCORE;
        }

        // 1. Resistance-Based Network Evaluation (primary signal)
        let resistance_score = ResistanceEvaluator::evaluate_for_player(board, player);

        // 2. Classical Heuristic Features (supplementary signal)
        let heuristic_score = Self::compute_heuristic_score(board, player);

        // 3. Blend: 75% resistance (accurate global, captures bridge chains),
        //    25% heuristic (tactical features, center control)
        let mut score = resistance_score * 0.75 + heuristic_score * 0.25;

        // 4. Bridge-Chain Threat Detection: penalize if opponent has a long
        //    bridge-connected chain spanning most of the board axis
        let opponent = if player == RED { BLUE } else { RED };
        let (opp_reach, _opp_chain_size) = Self::bridge_chain_reach(board, opponent);
        if opp_reach >= 0.5 {
            // Exponential penalty as opponent chain approaches full span
            // At 50% reach: -30, at 70%: -80, at 90%: -180, at 100%: -300
            let threat = (opp_reach - 0.4) * 500.0;
            score -= threat;
        }

        // 5. Bonus if our own chain is extensive
        let (my_reach, _my_chain_size) = Self::bridge_chain_reach(board, player);
        if my_reach >= 0.5 {
            let bonus = (my_reach - 0.4) * 300.0;
            score += bonus;
        }

        score
    }

    /// Usage:
    ///     let score = HexEvaluator::evaluate_fast(board, RED);
    /// Usage Example:
    ///     let eval = HexEvaluator::evaluate_fast(&board, current_player);
    /// Description:
    ///     Fast heuristic-only evaluation for deep search leaf nodes. Skips the expensive
    ///     resistance network solve. Used by pvs() at depth-0 for maximum throughput.
    #[inline(always)]
    pub fn evaluate_fast(board: &HexBoard, player: u8) -> f32 {
        let winner = board.get_winner();
        if winner == player {
            return WIN_SCORE;
        } else if winner != EMPTY {
            return -WIN_SCORE;
        }
        let mut score = Self::compute_heuristic_score(board, player);

        // Bridge-chain threat detection in fast eval too (critical for deep leaves)
        let opponent = if player == RED { BLUE } else { RED };
        let (opp_reach, _) = Self::bridge_chain_reach(board, opponent);
        if opp_reach >= 0.5 {
            score -= (opp_reach - 0.4) * 400.0;
        }
        let (my_reach, _) = Self::bridge_chain_reach(board, player);
        if my_reach >= 0.5 {
            score += (my_reach - 0.4) * 250.0;
        }

        score
    }

    /// Usage:
    ///     let score = HexEvaluator::compute_heuristic_score(board, player);
    /// Description:
    ///     Pure heuristic evaluation: shortest-path distance, bridge counting, center
    ///     control, corner quarantine, and central spine dominance. No resistance network.
    fn compute_heuristic_score(board: &HexBoard, player: u8) -> f32 {
        let opponent = if player == RED { BLUE } else { RED };
        let my_dist = Self::shortest_path(board, player) as i16;
        let opp_dist = Self::shortest_path(board, opponent) as i16;

        // Amplified threat scoring — virtual connections (dist=0) are near-wins
        let my_threat_score = match my_dist {
            0 => 800.0,  // Virtual connection = nearly won
            1 => 400.0,  // One stone from virtual connection
            2 => 200.0,
            3 => 100.0,
            4 => 50.0,
            d => (11.0 - d as f32).max(0.0) * 12.0,
        };

        let opp_threat_score = match opp_dist {
            0 => 800.0,  // Opponent virtual connection = nearly lost
            1 => 400.0,
            2 => 200.0,
            3 => 100.0,
            4 => 50.0,
            d => (11.0 - d as f32).max(0.0) * 12.0,
        };

        let mut score = my_threat_score - opp_threat_score;
        let size = board.size;

        // Acute Corner Quarantine & Neutralization
        if size > 2 {
            if board.get_cell(0, size - 1) == opponent && board.get_cell(0, size - 2) == player && board.get_cell(1, size - 2) == player && board.get_cell(1, size - 1) == player {
                score += 35.0;
            } else if board.get_cell(0, size - 1) == player && board.get_cell(0, size - 2) == opponent && board.get_cell(1, size - 2) == opponent && board.get_cell(1, size - 1) == opponent {
                score -= 35.0;
            }
            if board.get_cell(size - 1, 0) == opponent && board.get_cell(size - 1, 1) == player && board.get_cell(size - 2, 1) == player && board.get_cell(size - 2, 0) == player {
                score += 35.0;
            } else if board.get_cell(size - 1, 0) == player && board.get_cell(size - 1, 1) == opponent && board.get_cell(size - 2, 1) == opponent && board.get_cell(size - 2, 0) == opponent {
                score -= 35.0;
            }
        }

        // Bitboard Occupancy Iteration: Scan only active stones
        let total_stones = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;
        let center_mult = if total_stones <= 2 { 3.5 } else if total_stones <= 6 { 2.0 } else { 1.2 };

        let mut red_bits = board.red_bb.0;
        while red_bits != 0 {
            let idx = red_bits.trailing_zeros() as usize;
            red_bits &= red_bits - 1;
            let r = idx / size;
            let c = idx % size;
            let center_val = if size == 11 { CENTER_11[idx] } else { get_center_weight(r, c, size) };
            let stone_val = center_val * center_mult + evaluate_stone_bridges(board, r, c, RED, size);
            if player == RED { score += stone_val; } else { score -= stone_val; }
        }

        let mut blue_bits = board.blue_bb.0;
        while blue_bits != 0 {
            let idx = blue_bits.trailing_zeros() as usize;
            blue_bits &= blue_bits - 1;
            let r = idx / size;
            let c = idx % size;
            let center_val = if size == 11 { CENTER_11[idx] } else { get_center_weight(r, c, size) };
            let stone_val = center_val * center_mult + evaluate_stone_bridges(board, r, c, BLUE, size);
            if player == BLUE { score += stone_val; } else { score -= stone_val; }
        }

        // Central Crossing Spine & Transverse Wall Dominance
        let spine_score = evaluate_central_spine(board, size);
        if player == RED { score += spine_score; } else { score -= spine_score; }

        score
    }

    /// Usage:
    ///     let eval = HexEvaluator::evaluate_absolute(board);
    /// Usage Example:
    ///     let eval = HexEvaluator::evaluate_absolute(&board);
    /// Description:
    ///     Absolute evaluation from Red's perspective (+Red advantage, -Blue advantage).
    pub fn evaluate_absolute(board: &HexBoard) -> f32 {
        HexEvaluator::evaluate_for_player(board, RED)
    }

    /// Usage:
    ///     let wins = HexEvaluator::get_immediate_winning_moves(board, player);
    /// Usage Example:
    ///     let direct_wins = HexEvaluator::get_immediate_winning_moves(&board, RED);
    /// Description:
    ///     Uses SIMD bitboard flood-fill intersections to detect instant 1-move winning placements.
    pub fn get_immediate_winning_moves(board: &HexBoard, player: u8) -> Vec<(usize, usize)> {
        let size = board.size;
        let mut winning_moves = Vec::new();
        let (my_bb, top_mask, bottom_mask) = if player == RED {
            (&board.red_bb, Bitboard128::row_mask(0, size), Bitboard128::row_mask(size - 1, size))
        } else {
            (&board.blue_bb, Bitboard128::col_mask(0, size), Bitboard128::col_mask(size - 1, size))
        };

        if (my_bb.0 & top_mask.0) == 0 || (my_bb.0 & bottom_mask.0) == 0 {
            return winning_moves;
        }

        let mut top_front = Bitboard128(my_bb.0 & top_mask.0);
        while !top_front.is_empty() {
            let expanded = top_front.expand_neighbors(size);
            let next_bits = (expanded.0 & my_bb.0) & !top_front.0;
            if next_bits == 0 {
                break;
            }
            top_front.0 |= next_bits;
        }

        let mut bot_front = Bitboard128(my_bb.0 & bottom_mask.0);
        while !bot_front.is_empty() {
            let expanded = bot_front.expand_neighbors(size);
            let next_bits = (expanded.0 & my_bb.0) & !bot_front.0;
            if next_bits == 0 {
                break;
            }
            bot_front.0 |= next_bits;
        }

        let top_reach = top_front.expand_neighbors(size);
        let bot_reach = bot_front.expand_neighbors(size);
        let win_candidates = top_reach.0 & bot_reach.0 & !board.red_bb.0 & !board.blue_bb.0;

        if win_candidates != 0 {
            for r in 0..size {
                for c in 0..size {
                    let idx = r * size + c;
                    if (win_candidates & (1u128 << idx)) != 0 {
                        winning_moves.push((r, c));
                    }
                }
            }
        }

        winning_moves
    }

    /// Usage:
    ///     let moves = HexEvaluator::get_fast_promising_moves(board, player, 24);
    /// Usage Example:
    ///     let candidates = HexEvaluator::get_fast_promising_moves(&board, RED, 24);
    /// Description:
    ///     Ultra-fast move ordering for internal minimax tree nodes using SIMD bitboard neighbor expansions,
    ///     2-bridge formation, edge template connections, and opponent carrier disruption.
    #[inline(always)]
    pub fn get_fast_promising_moves(board: &HexBoard, player: u8, max_moves: usize) -> Vec<(usize, usize)> {
        let size = board.size;
        let mut scored_moves = Vec::with_capacity(size * size);
        let opponent = if player == RED { BLUE } else { RED };
        let my_bb = if player == RED { &board.red_bb } else { &board.blue_bb };
        let opp_bb = if player == RED { &board.blue_bb } else { &board.red_bb };

        let my_adj = my_bb.expand_neighbors(size);
        let opp_adj = opp_bb.expand_neighbors(size);

        let mut empty_bits = !(board.red_bb.0 | board.blue_bb.0);
        if size < 11 {
            let mut valid_mask = 0u128;
            for r in 0..size {
                for c in 0..size {
                    valid_mask |= 1u128 << (r * size + c);
                }
            }
            empty_bits &= valid_mask;
        } else if size == 11 {
            empty_bits &= (1u128 << 121) - 1;
        }

        while empty_bits != 0 {
            let idx = empty_bits.trailing_zeros() as usize;
            empty_bits &= empty_bits - 1;
            let r = idx / size;
            let c = idx % size;
            let bit = 1u128 << idx;

            let mut p = if size == 11 { CENTER_11[idx] } else { get_center_weight(r, c, size) };
            let ur = r as isize;
            let uc = c as isize;

            if (my_adj.0 & bit) != 0 {
                let mut my_count = 0;
                for k in 0..6 {
                    let nr = ur + DR[k];
                    let nc = uc + DC[k];
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        if board.get_cell(nr as usize, nc as usize) == player {
                            my_count += 1;
                        }
                    }
                }
                p += (my_count as f32) * 28.0;
                if my_count >= 2 {
                    p += 45.0;
                }
            }

            let mut opp_count = 0;
            if (opp_adj.0 & bit) != 0 {
                for k in 0..6 {
                    let nr = ur + DR[k];
                    let nc = uc + DC[k];
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        if board.get_cell(nr as usize, nc as usize) == opponent {
                            opp_count += 1;
                        }
                    }
                }
                p += (opp_count as f32) * 35.0;
                if opp_count >= 2 {
                    p += 65.0; // Stifling / corridor wedge bonus
                }
            }

            // 2-Bridge Link to Friendly Stone
            let mut friendly_2bridges = 0;
            for k in 0..6 {
                let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];
                let nr = ur + br;
                let nc = uc + bc;
                if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                    if board.get_cell(nr as usize, nc as usize) == player {
                        let cr1 = ur + c1r;
                        let cc1 = uc + c1c;
                        let cr2 = ur + c2r;
                        let cc2 = uc + c2c;
                        if cr1 >= 0 && cr1 < size as isize && cc1 >= 0 && cc1 < size as isize
                            && cr2 >= 0 && cr2 < size as isize && cc2 >= 0 && cc2 < size as isize
                        {
                            let c1 = board.get_cell(cr1 as usize, cc1 as usize);
                            let c2 = board.get_cell(cr2 as usize, cc2 as usize);
                            if c1 == EMPTY && c2 == EMPTY {
                                friendly_2bridges += 1;
                            } else if (c1 == opponent && c2 == EMPTY) || (c2 == opponent && c1 == EMPTY) {
                                p -= 70.0;
                            }
                        }
                    }
                }
            }
            if friendly_2bridges > 0 {
                p += 55.0 + (friendly_2bridges as f32 - 1.0) * 35.0;
            }

            // Opponent 2-Bridge Carrier Wedge Check: penalize futile attacks, reward genuine cuts
            for k in 0..6 {
                let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];
                let opp1_r = ur - c1r;
                let opp1_c = uc - c1c;
                let opp2_r = opp1_r + br;
                let opp2_c = opp1_c + bc;
                if opp1_r >= 0 && opp1_r < size as isize && opp1_c >= 0 && opp1_c < size as isize
                    && opp2_r >= 0 && opp2_r < size as isize && opp2_c >= 0 && opp2_c < size as isize
                {
                    if board.get_cell(opp1_r as usize, opp1_c as usize) == opponent
                        && board.get_cell(opp2_r as usize, opp2_c as usize) == opponent
                    {
                        let twin_r = opp1_r + c2r;
                        let twin_c = opp1_c + c2c;
                        let twin_cell = if twin_r >= 0 && twin_r < size as isize && twin_c >= 0 && twin_c < size as isize {
                            board.get_cell(twin_r as usize, twin_c as usize)
                        } else {
                            player
                        };
                        if twin_cell == player {
                            p += 90.0;
                        } else if twin_cell == EMPTY {
                            p -= 55.0;
                        }
                    }
                }

                let opp1_r2 = ur - c2r;
                let opp1_c2 = uc - c2c;
                let opp2_r2 = opp1_r2 + br;
                let opp2_c2 = opp1_c2 + bc;
                if opp1_r2 >= 0 && opp1_r2 < size as isize && opp1_c2 >= 0 && opp1_c2 < size as isize
                    && opp2_r2 >= 0 && opp2_r2 < size as isize && opp2_c2 >= 0 && opp2_c2 < size as isize
                {
                    if board.get_cell(opp1_r2 as usize, opp1_c2 as usize) == opponent
                        && board.get_cell(opp2_r2 as usize, opp2_c2 as usize) == opponent
                    {
                        let twin_r2 = opp1_r2 + c1r;
                        let twin_c2 = opp1_c2 + c1c;
                        let twin_cell2 = if twin_r2 >= 0 && twin_r2 < size as isize && twin_c2 >= 0 && twin_c2 < size as isize {
                            board.get_cell(twin_r2 as usize, twin_c2 as usize)
                        } else {
                            player
                        };
                        if twin_cell2 == player {
                            p += 90.0;
                        } else if twin_cell2 == EMPTY {
                            p -= 55.0;
                        }
                    }
                }
            }

            // Opponent 2-Bridge Frontier Interception (Preempting & Denying Opponent Expansion)
            let mut opp_frontiers_intercepted = 0;
            for k in 0..6 {
                let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];
                let opp_r = ur - br;
                let opp_c = uc - bc;
                if opp_r >= 0 && opp_r < size as isize && opp_c >= 0 && opp_c < size as isize {
                    if board.get_cell(opp_r as usize, opp_c as usize) == opponent {
                        let c1_r = opp_r + c1r;
                        let c1_c = opp_c + c1c;
                        let c2_r = opp_r + c2r;
                        let c2_c = opp_c + c2c;
                        if c1_r >= 0 && c1_r < size as isize && c1_c >= 0 && c1_c < size as isize
                            && c2_r >= 0 && c2_r < size as isize && c2_c >= 0 && c2_c < size as isize
                        {
                            if board.get_cell(c1_r as usize, c1_c as usize) == EMPTY
                                && board.get_cell(c2_r as usize, c2_c as usize) == EMPTY
                            {
                                opp_frontiers_intercepted += 1;
                                p += if opp_frontiers_intercepted == 1 { 80.0 } else { 45.0 };
                            }
                        }
                    }
                }
            }

            // Isolated / Detached Stone Penalty (Avoid detached rim moves like C3/E2/D9 while center is under siege)
            if (my_adj.0 & bit) == 0 && friendly_2bridges == 0 && (opp_adj.0 & bit) == 0 && opp_frontiers_intercepted == 0 {
                let total_my = if player == RED { board.red_bb.count_ones() } else { board.blue_bb.count_ones() };
                if total_my >= 2 {
                    p -= 80.0;
                }
            }

            // Isolated Rim Desertion Penalty
            let is_rim = if player == BLUE { c == 0 || c == size - 1 } else { r == 0 || r == size - 1 };
            if is_rim && (my_adj.0 & bit) == 0 && friendly_2bridges == 0 {
                p -= 95.0;
            }

            // Direct Edge Connection to Friendly Stone
            if player == RED {
                if r == 0 && (board.get_cell(1, c) == RED || (c > 0 && board.get_cell(1, c - 1) == RED)) {
                    p += 65.0;
                } else if r == size - 1 && (board.get_cell(size - 2, c) == RED || (c + 1 < size && board.get_cell(size - 2, c + 1) == RED)) {
                    p += 65.0;
                }
            } else {
                if c == 0 && (board.get_cell(r, 1) == BLUE || (r > 0 && board.get_cell(r - 1, 1) == BLUE)) {
                    p += 65.0;
                } else if c == size - 1 && (board.get_cell(r, size - 2) == BLUE || (r + 1 < size && board.get_cell(r + 1, size - 2) == BLUE)) {
                    p += 65.0;
                }
            }

            // Bridge-Chain Carrier Interception: if this cell is a carrier of an
            // opponent 2-bridge that's part of a long chain, boost heavily to disrupt it.
            // Check if this empty cell sits between two opponent stones as a carrier.
            let mut is_opp_bridge_carrier = false;
            for k in 0..6 {
                let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];
                // Check if (r,c) is carrier1 of a bridge between two opponent stones
                // Carrier1 position: stone_a at (r-c1r, c-c1c) and stone_b at (r-c1r+br, c-c1c+bc)
                let stone_a_r = ur - c1r;
                let stone_a_c = uc - c1c;
                let stone_b_r = stone_a_r + br;
                let stone_b_c = stone_a_c + bc;
                if stone_a_r >= 0 && stone_a_r < size as isize && stone_a_c >= 0 && stone_a_c < size as isize
                    && stone_b_r >= 0 && stone_b_r < size as isize && stone_b_c >= 0 && stone_b_c < size as isize
                {
                    if board.get_cell(stone_a_r as usize, stone_a_c as usize) == opponent
                        && board.get_cell(stone_b_r as usize, stone_b_c as usize) == opponent
                    {
                        // This cell is indeed carrier1 of an opponent bridge
                        // Check if carrier2 is still open (if so, this disruption matters)
                        let carrier2_r = stone_a_r + c2r;
                        let carrier2_c = stone_a_c + c2c;
                        if carrier2_r >= 0 && carrier2_r < size as isize
                            && carrier2_c >= 0 && carrier2_c < size as isize
                        {
                            if board.get_cell(carrier2_r as usize, carrier2_c as usize) == EMPTY {
                                is_opp_bridge_carrier = true;
                                p += 95.0; // Strong boost for disrupting opponent bridge
                            }
                        }
                    }
                }

                // Also check if (r,c) is carrier2
                let stone_a2_r = ur - c2r;
                let stone_a2_c = uc - c2c;
                let stone_b2_r = stone_a2_r + br;
                let stone_b2_c = stone_a2_c + bc;
                if stone_a2_r >= 0 && stone_a2_r < size as isize && stone_a2_c >= 0 && stone_a2_c < size as isize
                    && stone_b2_r >= 0 && stone_b2_r < size as isize && stone_b2_c >= 0 && stone_b2_c < size as isize
                {
                    if board.get_cell(stone_a2_r as usize, stone_a2_c as usize) == opponent
                        && board.get_cell(stone_b2_r as usize, stone_b2_c as usize) == opponent
                    {
                        let carrier1_r = stone_a2_r + c1r;
                        let carrier1_c = stone_a2_c + c1c;
                        if carrier1_r >= 0 && carrier1_r < size as isize
                            && carrier1_c >= 0 && carrier1_c < size as isize
                        {
                            if board.get_cell(carrier1_r as usize, carrier1_c as usize) == EMPTY {
                                is_opp_bridge_carrier = true;
                                p += 95.0;
                            }
                        }
                    }
                }
            }
            let _ = is_opp_bridge_carrier; // suppress unused warning

            scored_moves.push(((r, c), p));
        }

        scored_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_moves.into_iter().take(max_moves).map(|(m, _)| m).collect()
    }

    /// Usage:
    ///     let moves = HexEvaluator::get_promising_moves(board, player, 32);
    /// Usage Example:
    ///     let candidates = HexEvaluator::get_promising_moves(&board, RED, 24);
    /// Description:
    ///     Selects and prioritizes the highest-quality candidate moves using two-pass ordering:
    ///     fast pattern scoring for all candidates, then expensive path-aware re-scoring for top-N.
    pub fn get_promising_moves(board: &HexBoard, player: u8, max_moves: usize) -> Vec<(usize, usize)> {
        let size = board.size;
        let mut scored_moves = Vec::with_capacity(size * size);
        let opponent = if player == RED { BLUE } else { RED };

        let book_moves = crate::openings::HexOpeningBook::get_all_book_moves(board, player);

        for r in 0..size {
            for c in 0..size {
                if board.get_cell(r, c) != EMPTY {
                    continue;
                }

                let idx = r * size + c;
                let mut p = if size == 11 { CENTER_11[idx] } else { get_center_weight(r, c, size) };

                // 0. Opening Book Master Lines Priority
                if book_moves.contains(&(r, c)) {
                    p += 60.0;
                }

                // 1. Direct Neighbor Proximity & Joint Connections
                let mut my_count = 0;
                let mut opp_count = 0;
                for k in 0..6 {
                    let nr = r as isize + DR[k];
                    let nc = c as isize + DC[k];
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        let n_cell = board.get_cell(nr as usize, nc as usize);
                        if n_cell == player {
                            my_count += 1;
                            p += 28.0;
                        } else if n_cell == opponent {
                            opp_count += 1;
                            p += 35.0; // Stronger defensive / blocking score
                        }
                    }
                }
                if my_count >= 2 {
                    p += 45.0;
                }
                if opp_count >= 2 {
                    p += 75.0; // Active corridor stifle & shouldering
                }

                // 2. Fast Tactical Pattern Bonuses (2-bridges, edge templates, foils)
                p += HexPatternMatcher::evaluate_pattern_bonus(board, r, c, player);

                // 3. Opponent 2-Bridge & Edge Template Severing / Carrier Disruption
                let disruptions = HexPatternMatcher::count_opponent_carrier_disruptions(board, r, c, player);
                if disruptions > 0 {
                    p += (disruptions as f32) * 50.0;
                }

                // 4. Direct Edge Connection to Friendly Stone
                if player == RED {
                    if r == 0 && (board.get_cell(1, c) == RED || (c > 0 && board.get_cell(1, c - 1) == RED)) {
                        p += 75.0;
                    } else if r == size - 1 && (board.get_cell(size - 2, c) == RED || (c + 1 < size && board.get_cell(size - 2, c + 1) == RED)) {
                        p += 75.0;
                    }
                } else {
                    if c == 0 && (board.get_cell(r, 1) == BLUE || (r > 0 && board.get_cell(r - 1, 1) == BLUE)) {
                        p += 75.0;
                    } else if c == size - 1 && (board.get_cell(r, size - 2) == BLUE || (r + 1 < size && board.get_cell(r + 1, size - 2) == BLUE)) {
                        p += 75.0;
                    }
                }

                // 5. Central Spine & Crossing Contest Priority
                let mid = (size - 1) / 2;
                if size >= 7 && r >= mid.saturating_sub(2) && r <= mid + 2 && c >= mid.saturating_sub(2) && c <= mid + 2 {
                    p += 12.0;
                    if my_count >= 1 {
                        p += 18.0;
                    }
                }

                scored_moves.push(((r, c), p));
            }
        }

        scored_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 6. Expensive Path-Aware Re-scoring on Top-N Candidates Only
        let opp_dist = Self::shortest_path(board, opponent);
        let my_dist = Self::shortest_path(board, player);
        let need_opp_check = opp_dist <= 5;
        let need_own_check = my_dist <= 7 && my_dist > 0;
        let rescore_count = 40.min(scored_moves.len());

        if need_opp_check || need_own_check {
            let mut clone = board.clone();
            for i in 0..rescore_count {
                let ((r, c), ref mut p) = scored_moves[i];
                clone.place_move(r, c, player);

                if need_opp_check {
                    let opp_dist_after = Self::shortest_path(&clone, opponent);
                    if opp_dist_after > opp_dist {
                        *p += 110.0 * (4.0 - opp_dist as f32 + 1.0).max(1.0);
                    }
                }
                if need_own_check {
                    let my_dist_after = Self::shortest_path(&clone, player);
                    let improvement = my_dist - my_dist_after;
                    if improvement > 0 {
                        *p += (improvement as f32) * 90.0 + (7.0 - my_dist as f32).max(1.0) * 35.0;
                    }
                    if my_dist_after == 0 {
                        *p += 250.0; // Decisive virtual connection completion!
                    }
                }

                clone.undo_move();
            }
        }

        // Re-sort candidates after path re-scoring
        scored_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_moves.into_iter().take(max_moves).map(|(m, _)| m).collect()
    }
}
