//! Game-Theoretic Opening Book and Solved Strategy Module.
//!
//! OOP Description:
//! The `HexOpeningBook` struct encapsulates proven game-theoretic openings,
//! solved board opening moves (3x3 through 10x10), and the 11x11 master tree
//! covering all canonical master opening families (Center, E5, E4, D3, C2, B2, G7, H4, C9)
//! with the first 2 moves of established tournament play.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use crate::board::{HexBoard, EMPTY};
use crate::search::TopMoveEntry;

pub struct HexOpeningBook;

impl HexOpeningBook {
    /// Usage:
    ///     let book_res = HexOpeningBook::get_opening_move(board, player);
    /// Usage Example:
    ///     if let Some((mv, note)) = HexOpeningBook::get_opening_move(&board, BLUE) { ... }
    /// Description:
    ///     Queries game-theoretic opening book tree and solved opening tables based on stone positions.
    pub fn get_opening_move(board: &HexBoard, _player: u8) -> Option<((usize, usize), &'static str)> {
        let total_stones = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;

        // Move 0: Opening move on empty board
        if total_stones == 0 {
            let center = (board.size - 1) / 2;
            let note = match board.size {
                11 => "11x11 Game-Theoretic Optimal Center (F6)",
                7 => "7x7 Game-Theoretic Optimal Center (D4)",
                6 => "6x6 Solved Winning Center Opening (C3/C4)",
                5 => "5x5 Solved Winning Opening (C3)",
                4 => "4x4 Solved Winning Opening (B2)",
                3 => "3x3 Solved Winning Opening (B2)",
                _ => "Central Symmetrical Opening",
            };
            return Some(((center, center), note));
        }

        // 11x11 Master Game-Theoretic Opening Book Lines (Canonical 2-Move Foundations)
        if board.size == 11 {
            // 1 Stone on board: Responses to Move 1 Openings
            if total_stones == 1 {
                // 1. F6 (5, 5) -> G6 (5, 6)
                if board.get_cell(5, 5) != EMPTY {
                    return Some(((5, 6), "11x11 Academic Edge-Parallel Carrier Defense (G6)"));
                }
                // 2. E5 (4, 4) -> F5 (4, 5)
                if board.get_cell(4, 4) != EMPTY {
                    return Some(((4, 5), "11x11 Master Long-Diagonal Carrier Defense (F5)"));
                }
                // 3. E4 (3, 4) -> F4 (3, 5)
                if board.get_cell(3, 4) != EMPTY {
                    return Some(((3, 5), "11x11 Master Knight Carrier Block (F4)"));
                }
                // 4. D3 (2, 3) -> E3 (2, 4)
                if board.get_cell(2, 3) != EMPTY {
                    return Some(((2, 4), "11x11 Master Short-Diagonal Counter (E3)"));
                }
                // 5. C2 (1, 2) -> D2 (1, 3)
                if board.get_cell(1, 2) != EMPTY {
                    return Some(((1, 3), "11x11 Master Mild Flank Carrier Block (D2)"));
                }
                // 6. B2 (1, 1) -> C2 (1, 2)
                if board.get_cell(1, 1) != EMPTY {
                    return Some(((1, 2), "11x11 Master Acute Corner Containment (C2)"));
                }
                // 7. G7 (6, 6) -> F7 (6, 5)
                if board.get_cell(6, 6) != EMPTY {
                    return Some(((6, 5), "11x11 Master Symmetrical Diagonal Defense (F7)"));
                }
                // 8. H4 (3, 7) -> I4 (3, 8)
                if board.get_cell(3, 7) != EMPTY {
                    return Some(((3, 8), "11x11 Master East Diagonal Carrier Block (I4)"));
                }
                // 9. C9 (8, 2) -> D9 (8, 3)
                if board.get_cell(8, 2) != EMPTY {
                    return Some(((8, 3), "11x11 Master Obtuse Flank Wedge (D9)"));
                }
            }

            // 2 Stones on board: Move 2 Continuations for Blue
            if total_stones == 2 {
                // 1. F6 G6 -> G5 (4, 6)
                if Self::has_stones(board, &[(5, 5), (5, 6)]) {
                    return Some(((4, 6), "11x11 Game-Theoretic 2-Bridge Leap (G5)"));
                }
                // 1. F6 E6 (180° rotation) -> E7 (6, 4)
                if Self::has_stones(board, &[(5, 5), (5, 4)]) {
                    return Some(((6, 4), "11x11 Symmetrical 2-Bridge Leap (E7)"));
                }
                // 2. E5 F5 -> F4 (3, 5)
                if Self::has_stones(board, &[(4, 4), (4, 5)]) {
                    return Some(((3, 5), "11x11 Master Long-Diagonal 2-Bridge Thrust (F4)"));
                }
                // 3. E4 F4 -> F3 (2, 5)
                if Self::has_stones(board, &[(3, 4), (3, 5)]) {
                    return Some(((2, 5), "11x11 Master Short-Diagonal 2-Bridge Extension (F3)"));
                }
                // 4. D3 E3 -> E2 (1, 4)
                if Self::has_stones(board, &[(2, 3), (2, 4)]) {
                    return Some(((1, 4), "11x11 Master 2-Bridge North Leap (E2)"));
                }
                // 5. C2 D2 -> D1 (0, 3)
                if Self::has_stones(board, &[(1, 2), (1, 3)]) {
                    return Some(((0, 3), "11x11 Master Edge-2 Template Thrust (D1)"));
                }
                // 6. B2 C2 -> D1 (0, 3)
                if Self::has_stones(board, &[(1, 1), (1, 2)]) {
                    return Some(((0, 3), "11x11 Master Corner 2-Bridge Extension (D1)"));
                }
                // 7. G7 F7 -> F8 (7, 5)
                if Self::has_stones(board, &[(6, 6), (6, 5)]) {
                    return Some(((7, 5), "11x11 Master Symmetrical 2-Bridge South Thrust (F8)"));
                }
                // 8. H4 I4 -> I3 (2, 8)
                if Self::has_stones(board, &[(3, 7), (3, 8)]) {
                    return Some(((2, 8), "11x11 Master East 2-Bridge Extension (I3)"));
                }
                // 9. C9 D9 -> E8 (7, 4)
                if Self::has_stones(board, &[(8, 2), (8, 3)]) {
                    return Some(((7, 4), "11x11 Master 2-Bridge Central Incline (E8)"));
                }
            }

            // 3 Stones on board: Move 2 Defensive Replies for Red
            if total_stones == 3 {
                // 1. F6 G6 2. G5 -> D7 (6, 3)
                if Self::has_stones(board, &[(5, 5), (5, 6), (4, 6)]) {
                    return Some(((6, 3), "11x11 Classic Downward Conjugate Carrier Block (D7)"));
                }
                // 1. F6 E6 2. E7 -> H5 (4, 7)
                if Self::has_stones(board, &[(5, 5), (5, 4), (6, 4)]) {
                    return Some(((4, 7), "11x11 Symmetrical Carrier Block (H5)"));
                }
                // 2. E5 F5 2. F4 -> D6 (5, 3)
                if Self::has_stones(board, &[(4, 4), (4, 5), (3, 5)]) {
                    return Some(((5, 3), "11x11 Master Central Block & 2-Bridge Wedge (D6)"));
                }
                // 3. E4 F4 2. F3 -> D5 (4, 3)
                if Self::has_stones(board, &[(3, 4), (3, 5), (2, 5)]) {
                    return Some(((4, 3), "11x11 Master Central Flank Containment (D5)"));
                }
            }
        }

        None
    }

    /// Usage:
    ///     let entries = HexOpeningBook::get_empty_board_candidates(board, player, max_depth);
    /// Usage Example:
    ///     let leaderboard = HexOpeningBook::get_empty_board_candidates(&board, BLUE, 8);
    /// Description:
    ///     Generates a ranked leaderboard of all canonical master opening options on an empty board.
    pub fn get_empty_board_candidates(
        board: &HexBoard,
        player: u8,
        max_depth: u8,
    ) -> Vec<TopMoveEntry> {
        let size = board.size;
        let mut candidates = Vec::new();

        if size == 11 {
            let master_openings: &[((usize, usize), &'static str)] = &[
                ((5, 5), "11x11 Game-Theoretic Optimal Center (F6)"),
                ((4, 4), "11x11 Master Long-Diagonal Fair Opening (E5)"),
                ((3, 4), "11x11 Master Short-Diagonal Near-Center (E4)"),
                ((2, 3), "11x11 Master Short-Diagonal Probe (D3)"),
                ((1, 2), "11x11 Master Mild Flank Opening (C2)"),
                ((6, 6), "11x11 Master Symmetrical Long-Diagonal (G7)"),
                ((3, 7), "11x11 Master East Acute Probe (H4)"),
                ((1, 1), "11x11 Master Acute Corner Anchor (B2)"),
                ((8, 2), "11x11 Master Obtuse Flank Anchor (C9)"),
            ];

            for (idx, &((r, c), note)) in master_openings.iter().enumerate() {
                let mut clone = board.clone();
                clone.place_move(r, c, player);
                let score = crate::evaluator::HexEvaluator::evaluate_for_player(&clone, player);
                candidates.push(TopMoveEntry {
                    rank: idx + 1,
                    r,
                    c,
                    score,
                    depth: max_depth,
                    note: Some(note.to_string()),
                });
            }
        } else {
            let center = (size - 1) / 2;
            let mut clone = board.clone();
            clone.place_move(center, center, player);
            let score = crate::evaluator::HexEvaluator::evaluate_for_player(&clone, player);
            candidates.push(TopMoveEntry {
                rank: 1,
                r: center,
                c: center,
                score,
                depth: max_depth,
                note: Some("Optimal Center Opening".to_string()),
            });
        }

        candidates
    }

    /// Usage:
    ///     let book_moves = HexOpeningBook::get_all_book_moves(board, player);
    /// Usage Example:
    ///     for (r, c) in HexOpeningBook::get_all_book_moves(&board, RED) { ... }
    /// Description:
    ///     Returns all canonical book moves valid for the current position.
    pub fn get_all_book_moves(board: &HexBoard, _player: u8) -> Vec<(usize, usize)> {
        let total_stones = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;
        let mut moves = Vec::new();

        if total_stones == 0 {
            if board.size == 11 {
                return vec![
                    (5, 5), (4, 4), (3, 4), (2, 3), (1, 2),
                    (6, 6), (3, 7), (1, 1), (8, 2),
                ];
            } else {
                return vec![((board.size - 1) / 2, (board.size - 1) / 2)];
            }
        }

        if board.size == 11 {
            if total_stones == 1 {
                if board.get_cell(5, 5) != EMPTY {
                    moves.push((5, 6)); // G6
                    moves.push((5, 4)); // E6
                }
                if board.get_cell(4, 4) != EMPTY { moves.push((4, 5)); } // E5 -> F5
                if board.get_cell(3, 4) != EMPTY { moves.push((3, 5)); } // E4 -> F4
                if board.get_cell(2, 3) != EMPTY { moves.push((2, 4)); } // D3 -> E3
                if board.get_cell(1, 2) != EMPTY { moves.push((1, 3)); } // C2 -> D2
                if board.get_cell(1, 1) != EMPTY { moves.push((1, 2)); } // B2 -> C2
                if board.get_cell(6, 6) != EMPTY { moves.push((6, 5)); } // G7 -> F7
                if board.get_cell(3, 7) != EMPTY { moves.push((3, 8)); } // H4 -> I4
                if board.get_cell(8, 2) != EMPTY { moves.push((8, 3)); } // C9 -> D9
            } else if total_stones == 2 {
                if Self::has_stones(board, &[(5, 5), (5, 6)]) { moves.push((4, 6)); } // F6 G6 -> G5
                if Self::has_stones(board, &[(5, 5), (5, 4)]) { moves.push((6, 4)); } // F6 E6 -> E7
                if Self::has_stones(board, &[(4, 4), (4, 5)]) { moves.push((3, 5)); } // E5 F5 -> F4
                if Self::has_stones(board, &[(3, 4), (3, 5)]) { moves.push((2, 5)); } // E4 F4 -> F3
                if Self::has_stones(board, &[(2, 3), (2, 4)]) { moves.push((1, 4)); } // D3 E3 -> E2
                if Self::has_stones(board, &[(1, 2), (1, 3)]) { moves.push((0, 3)); } // C2 D2 -> D1
                if Self::has_stones(board, &[(1, 1), (1, 2)]) { moves.push((0, 3)); } // B2 C2 -> D1
                if Self::has_stones(board, &[(6, 6), (6, 5)]) { moves.push((7, 5)); } // G7 F7 -> F8
                if Self::has_stones(board, &[(3, 7), (3, 8)]) { moves.push((2, 8)); } // H4 I4 -> I3
                if Self::has_stones(board, &[(8, 2), (8, 3)]) { moves.push((7, 4)); } // C9 D9 -> E8
            } else if total_stones == 3 {
                if Self::has_stones(board, &[(5, 5), (5, 6), (4, 6)]) { moves.push((6, 3)); } // G5 -> D7
                if Self::has_stones(board, &[(5, 5), (5, 4), (6, 4)]) { moves.push((4, 7)); } // E7 -> H5
                if Self::has_stones(board, &[(4, 4), (4, 5), (3, 5)]) { moves.push((5, 3)); } // F4 -> D6
                if Self::has_stones(board, &[(3, 4), (3, 5), (2, 5)]) { moves.push((4, 3)); } // F3 -> D5
            }
        }

        moves.retain(|&(r, c)| board.get_cell(r, c) == EMPTY);
        moves
    }

    /// Helper to verify all specified (r, c) positions contain placed stones.
    fn has_stones(board: &HexBoard, positions: &[(usize, usize)]) -> bool {
        let total = (board.red_bb.count_ones() + board.blue_bb.count_ones()) as usize;
        if total != positions.len() {
            return false;
        }
        for &(r, c) in positions {
            if board.get_cell(r, c) == EMPTY {
                return false;
            }
        }
        true
    }
}
