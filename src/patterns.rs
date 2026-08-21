//! Hex Tactical Pattern, Ladder Escalation, Foil Interception, and Strategic Advisor.
//!
//! OOP Description:
//! The `HexPatternMatcher` struct identifies game-theoretic Hex tactical patterns,
//! including unescapable ladder runners, long-range ladder foils/interceptions,
//! compulsory 2-bridge carrier defenses, edge-2/edge-3 templates, and macro corridor flow analysis.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use crate::board::{HexBoard, BLUE, EMPTY, RED};

const DR: [isize; 6] = [-1, -1, 0, 0, 1, 1];
const DC: [isize; 6] = [0, 1, -1, 1, -1, 0];

const B2_OFFSETS: [(isize, isize, isize, isize, isize, isize); 6] = [
    (-2, 1, -1, 0, -1, 1),
    (-1, -1, -1, 0, 0, -1),
    (-1, 2, -1, 1, 0, 1),
    (1, -2, 0, -1, 1, -1),
    (1, 1, 0, 1, 1, 0),
    (2, -1, 1, -1, 1, 0),
];

#[derive(Clone, Copy, Debug)]
pub struct EdgeTemplateDef {
    pub dist: u8,
    pub rel_anchor_r: isize,
    pub rel_anchor_c: isize,
    pub carriers: &'static [(isize, isize)],
}

pub static RED_NORTH_TEMPLATES: &[EdgeTemplateDef] = &[
    // Edge-2 (dist 1): 2 carriers
    EdgeTemplateDef { dist: 1, rel_anchor_r: 1, rel_anchor_c: 0, carriers: &[(0, 0), (0, 1)] },
    EdgeTemplateDef { dist: 1, rel_anchor_r: 1, rel_anchor_c: 0, carriers: &[(0, -1), (0, 0)] },
    // Edge-3 (dist 2): 3 carriers
    EdgeTemplateDef { dist: 2, rel_anchor_r: 2, rel_anchor_c: 0, carriers: &[(1, 0), (1, 1), (0, 1)] },
    EdgeTemplateDef { dist: 2, rel_anchor_r: 2, rel_anchor_c: 0, carriers: &[(1, -1), (1, 0), (0, 0)] },
    // Edge-4 (dist 3): 5 carriers
    EdgeTemplateDef { dist: 3, rel_anchor_r: 3, rel_anchor_c: 0, carriers: &[(2, 0), (2, 1), (1, 1), (1, 2), (0, 2)] },
    EdgeTemplateDef { dist: 3, rel_anchor_r: 3, rel_anchor_c: 0, carriers: &[(2, -1), (2, 0), (1, -1), (1, 0), (0, -1)] },
    // Edge-5 (dist 4): 7 carriers
    EdgeTemplateDef { dist: 4, rel_anchor_r: 4, rel_anchor_c: 0, carriers: &[(3, 0), (3, 1), (2, 1), (2, 2), (1, 2), (1, 3), (0, 3)] },
];

pub static RED_SOUTH_TEMPLATES: &[EdgeTemplateDef] = &[
    // Edge-2 (dist 1): 2 carriers
    EdgeTemplateDef { dist: 1, rel_anchor_r: -1, rel_anchor_c: 0, carriers: &[(0, -1), (0, 0)] },
    EdgeTemplateDef { dist: 1, rel_anchor_r: -1, rel_anchor_c: 0, carriers: &[(0, 0), (0, 1)] },
    // Edge-3 (dist 2): 3 carriers
    EdgeTemplateDef { dist: 2, rel_anchor_r: -2, rel_anchor_c: 0, carriers: &[(-1, -1), (-1, 0), (0, -1)] },
    EdgeTemplateDef { dist: 2, rel_anchor_r: -2, rel_anchor_c: 0, carriers: &[(-1, 0), (-1, 1), (0, 0)] },
    // Edge-4 (dist 3): 5 carriers
    EdgeTemplateDef { dist: 3, rel_anchor_r: -3, rel_anchor_c: 0, carriers: &[(-2, -1), (-2, 0), (-1, -2), (-1, -1), (0, -2)] },
    EdgeTemplateDef { dist: 3, rel_anchor_r: -3, rel_anchor_c: 0, carriers: &[(-2, 0), (-2, 1), (-1, 0), (-1, 1), (0, 1)] },
    // Edge-5 (dist 4): 7 carriers
    EdgeTemplateDef { dist: 4, rel_anchor_r: -4, rel_anchor_c: 0, carriers: &[(-3, -1), (-3, 0), (-2, -2), (-2, -1), (-1, -3), (-1, -2), (0, -3)] },
];

pub static BLUE_WEST_TEMPLATES: &[EdgeTemplateDef] = &[
    // Edge-2 (dist 1): 2 carriers
    EdgeTemplateDef { dist: 1, rel_anchor_r: 0, rel_anchor_c: 1, carriers: &[(0, 0), (1, 0)] },
    EdgeTemplateDef { dist: 1, rel_anchor_r: 0, rel_anchor_c: 1, carriers: &[(-1, 0), (0, 0)] },
    // Edge-3 (dist 2): 3 carriers
    EdgeTemplateDef { dist: 2, rel_anchor_r: 0, rel_anchor_c: 2, carriers: &[(0, 1), (1, 1), (1, 0)] },
    EdgeTemplateDef { dist: 2, rel_anchor_r: 0, rel_anchor_c: 2, carriers: &[(-1, 1), (0, 1), (0, 0)] },
    // Edge-4 (dist 3): 5 carriers
    EdgeTemplateDef { dist: 3, rel_anchor_r: 0, rel_anchor_c: 3, carriers: &[(0, 2), (1, 2), (1, 1), (2, 1), (2, 0)] },
    EdgeTemplateDef { dist: 3, rel_anchor_r: 0, rel_anchor_c: 3, carriers: &[(-1, 2), (0, 2), (-1, 1), (0, 1), (-1, 0)] },
    // Edge-5 (dist 4): 7 carriers
    EdgeTemplateDef { dist: 4, rel_anchor_r: 0, rel_anchor_c: 4, carriers: &[(0, 3), (1, 3), (1, 2), (2, 2), (2, 1), (3, 1), (3, 0)] },
];

pub static BLUE_EAST_TEMPLATES: &[EdgeTemplateDef] = &[
    // Edge-2 (dist 1): 2 carriers
    EdgeTemplateDef { dist: 1, rel_anchor_r: 0, rel_anchor_c: -1, carriers: &[(-1, 0), (0, 0)] },
    EdgeTemplateDef { dist: 1, rel_anchor_r: 0, rel_anchor_c: -1, carriers: &[(0, 0), (1, 0)] },
    // Edge-3 (dist 2): 3 carriers
    EdgeTemplateDef { dist: 2, rel_anchor_r: 0, rel_anchor_c: -2, carriers: &[(-1, -1), (0, -1), (-1, 0)] },
    EdgeTemplateDef { dist: 2, rel_anchor_r: 0, rel_anchor_c: -2, carriers: &[(0, -1), (1, -1), (0, 0)] },
    // Edge-4 (dist 3): 5 carriers
    EdgeTemplateDef { dist: 3, rel_anchor_r: 0, rel_anchor_c: -3, carriers: &[(-1, -2), (0, -2), (-2, -1), (-1, -1), (-2, 0)] },
    EdgeTemplateDef { dist: 3, rel_anchor_r: 0, rel_anchor_c: -3, carriers: &[(0, -2), (1, -2), (0, -1), (1, -1), (1, 0)] },
    // Edge-5 (dist 4): 7 carriers
    EdgeTemplateDef { dist: 4, rel_anchor_r: 0, rel_anchor_c: -4, carriers: &[(-1, -3), (0, -3), (-2, -2), (-1, -2), (-3, -1), (-2, -1), (-3, 0)] },
];

#[derive(Clone, Debug)]
pub struct StrategicGuidance {
    pub intent: String,
    pub threat_level: u8,
    pub focus_area: String,
}

pub struct HexPatternMatcher;

impl HexPatternMatcher {
    /// Usage:
    ///     let connected = HexPatternMatcher::is_stone_connected_to_source_edge(board, r, c, player);
    /// Usage Example:
    ///     if HexPatternMatcher::is_stone_connected_to_source_edge(&board, 1, 3, RED) { ... }
    /// Description:
    ///     Checks if a stone is virtual-connected to the player's source edge (North for Red, West for Blue)
    ///     using precomputed Edge-2, Edge-3, Edge-4, and Edge-5 lookup tables.
    pub fn is_stone_connected_to_source_edge(board: &HexBoard, r: usize, c: usize, player: u8) -> bool {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };
        if player == RED {
            if r == 0 { return true; }
            if r > 2 { return false; }
            for tmpl in RED_NORTH_TEMPLATES {
                if tmpl.dist as usize == r {
                    let mut valid = true;
                    for &(cr, cc) in tmpl.carriers {
                        let actual_r = cr;
                        let actual_c = c as isize + cc;
                        if actual_r < 0 || actual_r >= size as isize || actual_c < 0 || actual_c >= size as isize {
                            valid = false;
                            break;
                        }
                        if board.get_cell(actual_r as usize, actual_c as usize) == opponent {
                            valid = false;
                            break;
                        }
                    }
                    if valid { return true; }
                }
            }
        } else {
            if c == 0 { return true; }
            if c > 2 { return false; }
            for tmpl in BLUE_WEST_TEMPLATES {
                if tmpl.dist as usize == c {
                    let mut valid = true;
                    for &(cr, cc) in tmpl.carriers {
                        let actual_r = r as isize + cr;
                        let actual_c = cc;
                        if actual_r < 0 || actual_r >= size as isize || actual_c < 0 || actual_c >= size as isize {
                            valid = false;
                            break;
                        }
                        if board.get_cell(actual_r as usize, actual_c as usize) == opponent {
                            valid = false;
                            break;
                        }
                    }
                    if valid { return true; }
                }
            }
        }
        false
    }

    /// Usage:
    ///     let connected = HexPatternMatcher::is_stone_connected_to_sink_edge(board, r, c, player);
    /// Usage Example:
    ///     if HexPatternMatcher::is_stone_connected_to_sink_edge(&board, 9, 3, RED) { ... }
    /// Description:
    ///     Checks if a stone is virtual-connected to the player's sink edge (South for Red, East for Blue)
    ///     using precomputed Edge-2 and Edge-3 lookup tables.
    pub fn is_stone_connected_to_sink_edge(board: &HexBoard, r: usize, c: usize, player: u8) -> bool {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };
        if player == RED {
            if r == size - 1 { return true; }
            let dist_from_sink = size - 1 - r;
            if dist_from_sink > 2 { return false; }
            for tmpl in RED_SOUTH_TEMPLATES {
                if tmpl.dist as usize == dist_from_sink {
                    let mut valid = true;
                    for &(cr, cc) in tmpl.carriers {
                        let actual_r = (size - 1) as isize + cr;
                        let actual_c = c as isize + cc;
                        if actual_r < 0 || actual_r >= size as isize || actual_c < 0 || actual_c >= size as isize {
                            valid = false;
                            break;
                        }
                        if board.get_cell(actual_r as usize, actual_c as usize) == opponent {
                            valid = false;
                            break;
                        }
                    }
                    if valid { return true; }
                }
            }
        } else {
            if c == size - 1 { return true; }
            let dist_from_sink = size - 1 - c;
            if dist_from_sink > 2 { return false; }
            for tmpl in BLUE_EAST_TEMPLATES {
                if tmpl.dist as usize == dist_from_sink {
                    let mut valid = true;
                    for &(cr, cc) in tmpl.carriers {
                        let actual_r = r as isize + cr;
                        let actual_c = (size - 1) as isize + cc;
                        if actual_r < 0 || actual_r >= size as isize || actual_c < 0 || actual_c >= size as isize {
                            valid = false;
                            break;
                        }
                        if board.get_cell(actual_r as usize, actual_c as usize) == opponent {
                            valid = false;
                            break;
                        }
                    }
                    if valid { return true; }
                }
            }
        }
        false
    }

    /// Usage:
    ///     let step = HexPatternMatcher::get_ladder_escape_move(board, player);
    /// Usage Example:
    ///     if let Some((r, c)) = HexPatternMatcher::get_ladder_escape_move(&board, RED) { ... }
    /// Description:
    ///     Detects whether opponent played a trailing ladder block and returns the forced forward ladder step.
    pub fn get_ladder_escape_move(board: &HexBoard, player: u8) -> Option<(usize, usize)> {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };

        if let Some(&(last_r, last_c, last_p)) = board.history.last() {
            if last_p != opponent {
                return None;
            }

            for k in 0..6 {
                let fr = last_r as isize + DR[k];
                let fc = last_c as isize + DC[k];
                if fr >= 0 && fr < size as isize && fc >= 0 && fc < size as isize {
                    if board.get_cell(fr as usize, fc as usize) == player {
                        let (target_r, target_c) = if player == RED {
                            if fr >= last_r as isize {
                                (fr + 1, fc)
                            } else {
                                (fr - 1, fc)
                            }
                        } else {
                            if fc >= last_c as isize {
                                (fr, fc + 1)
                            } else {
                                (fr, fc - 1)
                            }
                        };

                        if target_r >= 0 && target_r < size as isize && target_c >= 0 && target_c < size as isize {
                            if board.get_cell(target_r as usize, target_c as usize) == EMPTY {
                                return Some((target_r as usize, target_c as usize));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Usage:
    ///     let count = HexPatternMatcher::count_opponent_carrier_disruptions(board, r, c, player);
    /// Usage Example:
    ///     let disruptions = HexPatternMatcher::count_opponent_carrier_disruptions(&board, 5, 5, RED);
    /// Description:
    ///     Counts how many opponent 2-bridge virtual connections and edge templates are genuinely severed by playing at (r, c).
    ///     Only counts if the twin carrier is ALREADY occupied by player or off-board (so opponent cannot respond).
    pub fn count_opponent_carrier_disruptions(board: &HexBoard, r: usize, c: usize, player: u8) -> usize {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };
        let mut count = 0;
        let ur = r as isize;
        let uc = c as isize;

        // 1. Internal 2-bridge carrier disruption (checks both carrier 1 and carrier 2)
        for k in 0..6 {
            let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];

            // Case A: (r, c) is carrier 1
            let a1_r = ur - c1r;
            let a1_c = uc - c1c;
            let a2_r = a1_r + br;
            let a2_c = a1_c + bc;
            if a1_r >= 0 && a1_r < size as isize && a1_c >= 0 && a1_c < size as isize
                && a2_r >= 0 && a2_r < size as isize && a2_c >= 0 && a2_c < size as isize
            {
                if board.get_cell(a1_r as usize, a1_c as usize) == opponent
                    && board.get_cell(a2_r as usize, a2_c as usize) == opponent
                {
                    let twin_r = a1_r + c2r;
                    let twin_c = a1_c + c2c;
                    let twin_cell = if twin_r >= 0 && twin_r < size as isize && twin_c >= 0 && twin_c < size as isize {
                        board.get_cell(twin_r as usize, twin_c as usize)
                    } else {
                        player
                    };
                    if twin_cell == player {
                        count += 1;
                    }
                }
            }

            // Case B: (r, c) is carrier 2
            let a1_r2 = ur - c2r;
            let a1_c2 = uc - c2c;
            let a2_r2 = a1_r2 + br;
            let a2_c2 = a1_c2 + bc;
            if a1_r2 >= 0 && a1_r2 < size as isize && a1_c2 >= 0 && a1_c2 < size as isize
                && a2_r2 >= 0 && a2_r2 < size as isize && a2_c2 >= 0 && a2_c2 < size as isize
            {
                if board.get_cell(a1_r2 as usize, a1_c2 as usize) == opponent
                    && board.get_cell(a2_r2 as usize, a2_c2 as usize) == opponent
                {
                    let twin_r2 = a1_r2 + c1r;
                    let twin_c2 = a1_c2 + c1c;
                    let twin_cell2 = if twin_r2 >= 0 && twin_r2 < size as isize && twin_c2 >= 0 && twin_c2 < size as isize {
                        board.get_cell(twin_r2 as usize, twin_c2 as usize)
                    } else {
                        player
                    };
                    if twin_cell2 == player {
                        count += 1;
                    }
                }
            }
        }

        // 2. Edge template carrier disruption
        if opponent == RED {
            if r == 0 && c < size {
                if r + 1 < size && board.get_cell(1, c) == opponent && c + 1 < size && board.get_cell(0, c + 1) == player { count += 1; }
                if c > 0 && r + 1 < size && board.get_cell(1, c - 1) == opponent && board.get_cell(0, c - 1) == player { count += 1; }
            }
            if r == size - 1 && c < size {
                if r > 0 && board.get_cell(size - 2, c) == opponent && c + 1 < size && board.get_cell(size - 1, c + 1) == player { count += 1; }
                if c + 1 < size && r > 0 && board.get_cell(size - 2, c + 1) == opponent && c > 0 && board.get_cell(size - 1, c - 1) == player { count += 1; }
            }
        } else {
            if c == 0 && r < size {
                if c + 1 < size && board.get_cell(r, 1) == opponent && r + 1 < size && board.get_cell(r + 1, 0) == player { count += 1; }
                if r > 0 && c + 1 < size && board.get_cell(r - 1, 1) == opponent && board.get_cell(r - 1, 0) == player { count += 1; }
            }
            if c == size - 1 && r < size {
                if c > 0 && board.get_cell(r, size - 2) == opponent && r + 1 < size && board.get_cell(r + 1, size - 1) == player { count += 1; }
                if r + 1 < size && c > 0 && board.get_cell(r + 1, size - 2) == opponent && r > 0 && board.get_cell(r - 1, size - 1) == player { count += 1; }
            }
        }

        count
    }

    /// Usage:
    ///     let resp = HexPatternMatcher::get_compulsory_carrier_response(board, player);
    /// Usage Example:
    ///     if let Some((r, c)) = HexPatternMatcher::get_compulsory_carrier_response(&board, RED) { ... }
    /// Description:
    ///     Returns the compulsory twin carrier response when opponent attempts to sever an edge template or 2-bridge.
    pub fn get_compulsory_carrier_response(board: &HexBoard, player: u8) -> Option<(usize, usize)> {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };

        if let Some(&(last_r, last_c, last_p)) = board.history.last() {
            if last_p != opponent {
                return None;
            }

            let lr = last_r as isize;
            let lc = last_c as isize;

            // 1. Edge-2 Template Defenses for Red (North row 1 / South row N-2)
            if player == RED {
                if last_r == size - 1 {
                    let check_c1 = last_c;
                    let check_c2 = last_c + 1;

                    if check_c2 < size && board.get_cell(size - 2, check_c2) == RED {
                        if board.get_cell(size - 1, check_c2) == EMPTY {
                            return Some((size - 1, check_c2));
                        }
                    }
                    if check_c1 > 0 && board.get_cell(size - 2, check_c1) == RED {
                        if board.get_cell(size - 1, check_c1 - 1) == EMPTY {
                            return Some((size - 1, check_c1 - 1));
                        }
                    }
                }
                if last_r == 0 {
                    let check_c1 = last_c;
                    let check_c2 = if last_c > 0 { last_c - 1 } else { 0 };

                    if board.get_cell(1, check_c1) == RED && last_c + 1 < size {
                        if board.get_cell(0, last_c + 1) == EMPTY {
                            return Some((0, last_c + 1));
                        }
                    }
                    if last_c > 0 && board.get_cell(1, check_c2) == RED {
                        if board.get_cell(0, last_c - 1) == EMPTY {
                            return Some((0, last_c - 1));
                        }
                    }
                }
            } else {
                // 2. Edge-2 Template Defenses for Blue (West col 1 / East col N-2)
                if last_c == size - 1 {
                    let check_r2 = last_r + 1;
                    let check_r1 = last_r;

                    if check_r2 < size && board.get_cell(check_r2, size - 2) == BLUE {
                        if board.get_cell(check_r2, size - 1) == EMPTY {
                            return Some((check_r2, size - 1));
                        }
                    }
                    if check_r1 > 0 && board.get_cell(check_r1, size - 2) == BLUE {
                        if board.get_cell(check_r1 - 1, size - 1) == EMPTY {
                            return Some((check_r1 - 1, size - 1));
                        }
                    }
                }
                if last_c == 0 {
                    if last_r + 1 < size && board.get_cell(last_r, 1) == BLUE {
                        if board.get_cell(last_r + 1, 0) == EMPTY {
                            return Some((last_r + 1, 0));
                        }
                    }
                    if last_r > 0 && board.get_cell(last_r - 1, 1) == BLUE {
                        if board.get_cell(last_r - 1, 0) == EMPTY {
                            return Some((last_r - 1, 0));
                        }
                    }
                }
            }

            // 3. Internal 2-Bridge Carriers (symmetric for both carrier 1 and carrier 2 attacks)
            for k in 0..6 {
                let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];

                // Case A: opponent played into carrier 1
                let anchor_r = lr - c1r;
                let anchor_c = lc - c1c;
                let target_r = anchor_r + br;
                let target_c = anchor_c + bc;

                if anchor_r >= 0 && anchor_r < size as isize && anchor_c >= 0 && anchor_c < size as isize
                    && target_r >= 0 && target_r < size as isize && target_c >= 0 && target_c < size as isize
                {
                    if board.get_cell(anchor_r as usize, anchor_c as usize) == player
                        && board.get_cell(target_r as usize, target_c as usize) == player
                    {
                        let twin_r = anchor_r + c2r;
                        let twin_c = anchor_c + c2c;
                        if twin_r >= 0 && twin_r < size as isize && twin_c >= 0 && twin_c < size as isize {
                            if board.get_cell(twin_r as usize, twin_c as usize) == EMPTY {
                                return Some((twin_r as usize, twin_c as usize));
                            }
                        }
                    }
                }

                // Case B: opponent played into carrier 2
                let anchor_r2 = lr - c2r;
                let anchor_c2 = lc - c2c;
                let target_r2 = anchor_r2 + br;
                let target_c2 = anchor_c2 + bc;

                if anchor_r2 >= 0 && anchor_r2 < size as isize && anchor_c2 >= 0 && anchor_c2 < size as isize
                    && target_r2 >= 0 && target_r2 < size as isize && target_c2 >= 0 && target_c2 < size as isize
                {
                    if board.get_cell(anchor_r2 as usize, anchor_c2 as usize) == player
                        && board.get_cell(target_r2 as usize, target_c2 as usize) == player
                    {
                        let twin_r2 = anchor_r2 + c1r;
                        let twin_c2 = anchor_c2 + c1c;
                        if twin_r2 >= 0 && twin_r2 < size as isize && twin_c2 >= 0 && twin_c2 < size as isize {
                            if board.get_cell(twin_r2 as usize, twin_c2 as usize) == EMPTY {
                                return Some((twin_r2 as usize, twin_c2 as usize));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Usage:
    ///     let is_trailing = HexPatternMatcher::is_trailing_ladder_push(board, r, c, player);
    /// Usage Example:
    ///     if HexPatternMatcher::is_trailing_ladder_push(&board, 1, 3, BLUE) { ... }
    /// Description:
    ///     Identifies if a move is a futile trailing ladder block against an unblockable runner.
    pub fn is_trailing_ladder_push(board: &HexBoard, r: usize, c: usize, player: u8) -> bool {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };
        let ur = r as isize;
        let uc = c as isize;

        for k in 0..6 {
            let nr = ur + DR[k];
            let nc = uc + DC[k];
            if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                if board.get_cell(nr as usize, nc as usize) == opponent {
                    if opponent == RED {
                        if nr <= (size / 2) as isize && nr > 0 {
                            let fwd_r = nr - 1;
                            let fwd_c = nc;
                            if board.get_cell(fwd_r as usize, fwd_c as usize) == EMPTY && uc != fwd_c {
                                return true;
                            }
                        }
                        if nr >= (size / 2) as isize && nr < (size - 1) as isize {
                            let fwd_r = nr + 1;
                            let fwd_c = nc;
                            if board.get_cell(fwd_r as usize, fwd_c as usize) == EMPTY && uc != fwd_c {
                                return true;
                            }
                        }
                    } else {
                        if nc <= (size / 2) as isize && nc > 0 {
                            let fwd_r = nr;
                            let fwd_c = nc - 1;
                            if board.get_cell(fwd_r as usize, fwd_c as usize) == EMPTY && ur != fwd_r {
                                return true;
                            }
                        }
                        if nc >= (size / 2) as isize && nc < (size - 1) as isize {
                            let fwd_r = nr;
                            let fwd_c = nc + 1;
                            if board.get_cell(fwd_r as usize, fwd_c as usize) == EMPTY && ur != fwd_r {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Usage:
    ///     let bonus = HexPatternMatcher::evaluate_pattern_bonus(board, r, c, player);
    /// Usage Example:
    ///     let score = HexPatternMatcher::evaluate_pattern_bonus(&board, 7, 3, RED);
    /// Description:
    ///     Evaluates tactical pattern bonuses including 2-bridge leaps, edge templates, long-range foils, and carrier wedges.
    pub fn evaluate_pattern_bonus(board: &HexBoard, r: usize, c: usize, player: u8) -> f32 {
        let size = board.size;
        let opponent = if player == RED { BLUE } else { RED };
        let mut bonus = 0.0f32;

        let ur = r as isize;
        let uc = c as isize;

        // 1. Heavy Penalty for Futile Trailing Ladder Blocks
        if Self::is_trailing_ladder_push(board, r, c, player) {
            bonus -= 85.0;
        }

        // 2. Long-Range Ladder Foil & Edge Interception (Playing Ahead in Corridor)
        // Only grant foil bonus if the opponent has an active multi-stone runner heading toward this rim (within 3 cells)
        if player == BLUE {
            if r <= 1 {
                let mut opp_near = 0;
                for check_r in 2..=4.min(size - 1) {
                    if board.get_cell(check_r, c) == RED {
                        opp_near += 1;
                    }
                }
                if opp_near >= 2 {
                    bonus += 55.0;
                }
            } else if r >= size - 2 {
                let mut opp_near = 0;
                for check_r in (size.saturating_sub(5))..=(size.saturating_sub(3)) {
                    if board.get_cell(check_r, c) == RED {
                        opp_near += 1;
                    }
                }
                if opp_near >= 2 {
                    bonus += 55.0;
                }
            }
        } else {
            if c <= 1 {
                let mut opp_near = 0;
                for check_c in 2..=4.min(size - 1) {
                    if board.get_cell(r, check_c) == BLUE {
                        opp_near += 1;
                    }
                }
                if opp_near >= 2 {
                    bonus += 55.0;
                }
            } else if c >= size - 2 {
                let mut opp_near = 0;
                for check_c in (size.saturating_sub(5))..=(size.saturating_sub(3)) {
                    if board.get_cell(r, check_c) == BLUE {
                        opp_near += 1;
                    }
                }
                if opp_near >= 2 {
                    bonus += 55.0;
                }
            }
        }

        // 3. Multi-Bridge Mesh Forks & 2-Bridge Links to Friendly Stones
        let mut friendly_bridges = 0;
        for k in 0..6 {
            let (br, bc, c1r, c1c, c2r, c2c) = B2_OFFSETS[k];
            let nr = ur + br;
            let nc = uc + bc;
            if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                let n_cell = board.get_cell(nr as usize, nc as usize);
                if n_cell == player {
                    let car1_r = ur + c1r;
                    let car1_c = uc + c1c;
                    let car2_r = ur + c2r;
                    let car2_c = uc + c2c;
                    if car1_r >= 0 && car1_r < size as isize && car1_c >= 0 && car1_c < size as isize
                        && car2_r >= 0 && car2_r < size as isize && car2_c >= 0 && car2_c < size as isize
                    {
                        let c1 = board.get_cell(car1_r as usize, car1_c as usize);
                        let c2 = board.get_cell(car2_r as usize, car2_c as usize);
                        if c1 == EMPTY && c2 == EMPTY {
                            friendly_bridges += 1;
                            bonus += if friendly_bridges == 1 { 90.0 } else { 60.0 };
                        } else if (c1 == opponent && c2 == EMPTY) || (c2 == opponent && c1 == EMPTY) {
                            // Contested 2-bridge blunder: opponent occupies 1 carrier; playing this invites opponent to seize the other carrier and complete their own connection!
                            bonus -= 70.0;
                        } else if c1 != opponent && c2 != opponent {
                            bonus += 25.0;
                        }
                    }
                }
            }

            // 4. Carrier Wedge (Disrupting & Severing Opponent 2-Bridge Chains)
            // Case A: (r, c) is carrier 1
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
                        bonus += 110.0;
                    } else if twin_cell == EMPTY {
                        bonus -= 65.0; // Futile attack into unbroken 2-bridge
                    }
                }
            }

            // Case B: (r, c) is carrier 2
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
                        bonus += 110.0;
                    } else if twin_cell2 == EMPTY {
                        bonus -= 65.0; // Futile attack into unbroken 2-bridge
                    }
                }
            }
        }

        // 5. Opponent 2-Bridge Frontier Interception (Preempting & Denying Opponent Expansion)
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
                            // Playing (r, c) directly denies opponent from establishing a 2-bridge from opp_r, opp_c!
                            opp_frontiers_intercepted += 1;
                            bonus += if opp_frontiers_intercepted == 1 { 85.0 } else { 50.0 };
                        }
                    }
                }
            }
        }

        // 6. Precomputed Edge Templates (Edge-2, Edge-3, Edge-4, Edge-5)
        let my_stones = if player == RED { board.red_bb.count_ones() } else { board.blue_bb.count_ones() };
        let mut has_edge_template = false;
        if my_stones > 0 {
            if Self::is_stone_connected_to_source_edge(board, r, c, player) {
                bonus += 85.0;
                has_edge_template = true;
            }
            if Self::is_stone_connected_to_sink_edge(board, r, c, player) {
                bonus += 85.0;
                has_edge_template = true;
            }
        }

        // 7. Isolated / Detached Stone Penalty (Avoid desertion moves like E2/C3/D9 while center is burning)
        let has_opp_adj = {
            let mut found = false;
            for k in 0..6 {
                let nr = ur + DR[k];
                let nc = uc + DC[k];
                if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                    if board.get_cell(nr as usize, nc as usize) == opponent {
                        found = true;
                        break;
                    }
                }
            }
            found
        };

        if friendly_bridges == 0 && !has_edge_template && opp_frontiers_intercepted == 0 && !has_opp_adj {
            let my_adj_count = {
                let mut c = 0;
                for k in 0..6 {
                    let nr = ur + DR[k];
                    let nc = uc + DC[k];
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        if board.get_cell(nr as usize, nc as usize) == player {
                            c += 1;
                        }
                    }
                }
                c
            };
            if my_adj_count == 0 && my_stones >= 2 {
                bonus -= 80.0; // Heavy penalty for completely detached moves away from the active battle
            }
        }

        // 6. Border Cutoff Wall Containment Defense (SIMD Bitboards)
        if player == BLUE {
            let opp_bb = board.red_bb.0;
            let mut opp_west = 0;
            let mut opp_east = 0;
            for col in 0..=2.min(size - 1) {
                opp_west += (opp_bb & crate::bitboard::Bitboard128::col_mask(col, size).0).count_ones() as usize;
            }
            for col in (size.saturating_sub(3))..size {
                opp_east += (opp_bb & crate::bitboard::Bitboard128::col_mask(col, size).0).count_ones() as usize;
            }
            if opp_west >= 3 && c <= 3 {
                bonus += 70.0;
            }
            if opp_east >= 3 && c >= size - 4 {
                bonus += 70.0;
            }
        } else {
            let opp_bb = board.blue_bb.0;
            let mut opp_north = 0;
            let mut opp_south = 0;
            for row in 0..=2.min(size - 1) {
                opp_north += (opp_bb & crate::bitboard::Bitboard128::row_mask(row, size).0).count_ones() as usize;
            }
            for row in (size.saturating_sub(3))..size {
                opp_south += (opp_bb & crate::bitboard::Bitboard128::row_mask(row, size).0).count_ones() as usize;
            }
            if opp_north >= 3 && r <= 3 {
                bonus += 70.0;
            }
            if opp_south >= 3 && r >= size - 4 {
                bonus += 70.0;
            }
        }

        // 7. Acute Corner Isolation & Sealing (K1 / A11 Corner Defenses)
        if (r == 0 && c == size.saturating_sub(2))
            || (r == 1 && c == size.saturating_sub(2))
            || (r == 1 && c == size.saturating_sub(1))
        {
            if board.get_cell(0, size - 1) == opponent {
                let mut friendly_seal = 0;
                if board.get_cell(0, size - 2) == player { friendly_seal += 1; }
                if board.get_cell(1, size - 2) == player { friendly_seal += 1; }
                if board.get_cell(1, size - 1) == player { friendly_seal += 1; }
                bonus += 65.0 + if friendly_seal >= 2 { 85.0 } else { 35.0 };
            }
        }
        if (r == size.saturating_sub(1) && c == 1)
            || (r == size.saturating_sub(2) && c == 1)
            || (r == size.saturating_sub(2) && c == 0)
        {
            if board.get_cell(size - 1, 0) == opponent {
                let mut friendly_seal = 0;
                if board.get_cell(size - 1, 1) == player { friendly_seal += 1; }
                if board.get_cell(size - 2, 1) == player { friendly_seal += 1; }
                if board.get_cell(size - 2, 0) == player { friendly_seal += 1; }
                bonus += 65.0 + if friendly_seal >= 2 { 85.0 } else { 35.0 };
            }
        }

        // 8. Isolated Edge Desertion Penalty
        let is_edge_flank = if player == BLUE { c <= 1 || c >= size - 2 } else { r <= 1 || r >= size - 2 };
        if is_edge_flank {
            let my_bb = if player == BLUE { &board.blue_bb } else { &board.red_bb };
            let my_adj = my_bb.expand_neighbors(size);
            if (my_adj.0 & (1u128 << (r * size + c))) == 0 && friendly_bridges == 0 {
                bonus -= 95.0;
            }
        }

        bonus
    }

    /// Usage:
    ///     let path_bonus = HexPatternMatcher::evaluate_path_bonus(board, r, c, player);
    /// Usage Example:
    ///     let path_bonus = HexPatternMatcher::evaluate_path_bonus(&board, 3, 9, BLUE);
    /// Description:
    ///     Computes expensive path-aware tactical bonuses (opponent interception + own-path shortening).
    ///     Should only be called on top-N candidates after fast pattern scoring to limit clone+shortest_path overhead.
    pub fn evaluate_path_bonus(board: &HexBoard, r: usize, c: usize, player: u8) -> f32 {
        let opponent = if player == RED { BLUE } else { RED };
        let mut bonus = 0.0f32;

        let opp_dist = crate::evaluator::HexEvaluator::shortest_path(board, opponent);
        let my_dist = crate::evaluator::HexEvaluator::shortest_path(board, player);
        let need_opp_check = opp_dist <= 2;
        let need_own_check = my_dist <= 4 && my_dist > 0;

        if need_opp_check || need_own_check {
            let mut clone = board.clone();
            clone.place_move(r, c, player);

            if need_opp_check {
                let opp_dist_after = crate::evaluator::HexEvaluator::shortest_path(&clone, opponent);
                if opp_dist_after > opp_dist {
                    bonus += 110.0 * (3.0 - opp_dist as f32 + 1.0);
                }
            }

            if need_own_check {
                let my_dist_after = crate::evaluator::HexEvaluator::shortest_path(&clone, player);
                let improvement = my_dist - my_dist_after;
                if improvement > 0 {
                    bonus += (improvement as f32) * 70.0 + (5.0 - my_dist as f32) * 25.0;
                }
            }
        }

        bonus
    }

    /// Usage:
    ///     let pb = HexPatternMatcher::evaluate_path_bonus_with_dists(board, r, c, player, opp_dist, my_dist);
    /// Usage Example:
    ///     let pb = HexPatternMatcher::evaluate_path_bonus_with_dists(&board, 3, 9, BLUE, 2, 3);
    /// Description:
    ///     Path-aware tactical bonuses with precomputed base distances to avoid redundant BFS.
    ///     Used by two-pass move ordering to score top-N candidates efficiently.
    pub fn evaluate_path_bonus_with_dists(
        board: &HexBoard, r: usize, c: usize, player: u8,
        opp_dist: i16, my_dist: i16,
    ) -> f32 {
        let opponent = if player == RED { BLUE } else { RED };
        let mut bonus = 0.0f32;

        let need_opp_check = opp_dist <= 2;
        let need_own_check = my_dist <= 4 && my_dist > 0;

        if need_opp_check || need_own_check {
            let mut clone = board.clone();
            clone.place_move(r, c, player);

            if need_opp_check {
                let opp_dist_after = crate::evaluator::HexEvaluator::shortest_path(&clone, opponent);
                if opp_dist_after > opp_dist {
                    bonus += 110.0 * (3.0 - opp_dist as f32 + 1.0);
                }
            }

            if need_own_check {
                let my_dist_after = crate::evaluator::HexEvaluator::shortest_path(&clone, player);
                let improvement = my_dist - my_dist_after;
                if improvement > 0 {
                    bonus += (improvement as f32) * 70.0 + (5.0 - my_dist as f32) * 25.0;
                }
            }
        }

        bonus
    }

    /// Usage:
    ///     let guide = HexPatternMatcher::get_strategic_guidance(board, player);
    /// Usage Example:
    ///     let plan = HexPatternMatcher::get_strategic_guidance(&board, RED);
    /// Description:
    ///     Synthesizes high-level macro strategy, opponent threat radar, and corridor goals.
    pub fn get_strategic_guidance(board: &HexBoard, player: u8) -> StrategicGuidance {
        let opponent = if player == RED { BLUE } else { RED };
        let my_dist = crate::evaluator::HexEvaluator::shortest_path(board, player);
        let opp_dist = crate::evaluator::HexEvaluator::shortest_path(board, opponent);
        let size = board.size;

        let mut siege_area = None;
        if player == BLUE {
            let mut opp_west = 0;
            for r in 0..size {
                for c in 0..=2.min(size - 1) {
                    if board.get_cell(r, c) == opponent { opp_west += 1; }
                }
            }
            if opp_west >= 3 {
                siege_area = Some("West Border Gateways");
            }
        } else {
            let mut opp_north = 0;
            for r in 0..=2.min(size - 1) {
                for c in 0..size {
                    if board.get_cell(r, c) == opponent { opp_north += 1; }
                }
            }
            if opp_north >= 3 {
                siege_area = Some("North Border Gateways");
            }
        }

        let (intent, threat_level, focus_area) = if let Some(area) = siege_area {
            ("CRITICAL: Defend border cutoff wall & secure breach corridor!".to_string(), 3, area.to_string())
        } else if opp_dist <= 2 {
            ("URGENT: Block immediate opponent edge connection!".to_string(), 3, "Border Gateways".to_string())
        } else if my_dist <= 3 {
            ("Advancing decisive winning bridge to border.".to_string(), 1, "Goal Corridor".to_string())
        } else if opp_dist < my_dist {
            ("Intercept opponent forward ladder ahead of corridor.".to_string(), 2, "Chokepoints".to_string())
        } else {
            ("Develop central 2-bridge network and flank templates.".to_string(), 1, "Board Center".to_string())
        };

        StrategicGuidance {
            intent,
            threat_level,
            focus_area,
        }
    }
}
