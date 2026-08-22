//! Hex Board State and Move Representation Module.
//!
//! OOP Description:
//! The `HexBoard` struct encapsulates the hexagonal game state, maintaining
//! Red and Blue SIMD bitboards, move history, turn management, and 64-bit Zobrist hashes
//! with player-turn discrimination. It supports sub-microsecond move placement, undo operations,
//! and ultra-fast SIMD terminal win detection.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use crate::bitboard::Bitboard128;

pub const EMPTY: u8 = 0;
pub const RED: u8 = 1;
pub const BLUE: u8 = 2;

lazy_static::lazy_static! {
    static ref ZOBRIST_TABLE: [[u64; 3]; 196] = {
        let mut table = [[0u64; 3]; 196];
        let mut state = 0x853c49e6748fea9bu64;
        for i in 0..196 {
            for p in 0..3 {
                state ^= state >> 12;
                state ^= state << 25;
                state ^= state >> 27;
                table[i][p] = state.wrapping_mul(0x2545F4914F6CDD1Du64);
            }
        }
        table
    };
    static ref ZOBRIST_PLAYER: [u64; 3] = [0, 0x123456789ABCDEF0, 0xFEDCBA9876543210];
}

/// Usage:
///     let key = zobrist_key(idx, player);
/// Usage Example:
///     let key = zobrist_key(55, RED);
/// Description:
///     Returns the Zobrist key for a given cell index and player, for incremental hash computation.
#[inline(always)]
pub fn zobrist_key(idx: usize, player: u8) -> u64 {
    ZOBRIST_TABLE[idx][player as usize]
}

/// Usage:
///     let key = zobrist_player_key(player);
/// Usage Example:
///     let key = zobrist_player_key(BLUE);
/// Description:
///     Returns the Zobrist player-turn key for toggling active player in incremental hashing.
#[inline(always)]
pub fn zobrist_player_key(player: u8) -> u64 {
    ZOBRIST_PLAYER[player as usize]
}

#[derive(Clone, Debug)]
pub struct HexBoard {
    pub size: usize,
    pub red_bb: Bitboard128,
    pub blue_bb: Bitboard128,
    pub current_player: u8,
    pub history: Vec<(usize, usize, u8)>,
    pub zobrist_hash: u64,
}

impl HexBoard {
    /// Usage:
    ///     let board = HexBoard::new(11);
    /// Usage Example:
    ///     let board = HexBoard::new(11);
    ///     assert_eq!(board.size, 11);
    /// Description:
    ///     Initializes an empty Hex board with given dimension and sets active player to Blue.
    pub fn new(size: usize) -> Self {
        HexBoard {
            size,
            red_bb: Bitboard128::empty(),
            blue_bb: Bitboard128::empty(),
            current_player: BLUE,
            history: Vec::with_capacity(size * size),
            zobrist_hash: ZOBRIST_PLAYER[BLUE as usize],
        }
    }

    /// Usage:
    ///     board.set_current_player(RED);
    /// Usage Example:
    ///     board.set_current_player(player);
    /// Description:
    ///     Sets active player turn and updates Zobrist hash if different from current turn.
    #[inline(always)]
    pub fn set_current_player(&mut self, player: u8) {
        if self.current_player != player {
            self.zobrist_hash ^= ZOBRIST_PLAYER[self.current_player as usize] ^ ZOBRIST_PLAYER[player as usize];
            self.current_player = player;
        }
    }

    /// Usage:
    ///     let cell_val = board.get_cell(r, c);
    /// Usage Example:
    ///     if board.get_cell(5, 5) == EMPTY { ... }
    /// Description:
    ///     Returns state of cell (EMPTY, RED, or BLUE) at coordinates (r, c).
    #[inline(always)]
    pub fn get_cell(&self, r: usize, c: usize) -> u8 {
        if self.red_bb.has_bit(r, c, self.size) {
            RED
        } else if self.blue_bb.has_bit(r, c, self.size) {
            BLUE
        } else {
            EMPTY
        }
    }

    /// Usage:
    ///     board.place_move(r, c, player);
    /// Usage Example:
    ///     board.place_move(5, 5, BLUE);
    /// Description:
    ///     Places stone for player, updates bitboards and Zobrist hash, and toggles active turn.
    #[inline(always)]
    pub fn place_move(&mut self, r: usize, c: usize, player: u8) -> bool {
        if self.get_cell(r, c) != EMPTY {
            return false;
        }

        let idx = r * self.size + c;
        let next_player = if player == RED { BLUE } else { RED };

        if player == RED {
            self.red_bb.set_bit(r, c, self.size);
        } else {
            self.blue_bb.set_bit(r, c, self.size);
        }

        self.zobrist_hash ^= ZOBRIST_TABLE[idx][player as usize];
        self.zobrist_hash ^= ZOBRIST_PLAYER[player as usize] ^ ZOBRIST_PLAYER[next_player as usize];
        self.history.push((r, c, player));
        self.current_player = next_player;
        true
    }

    /// Usage:
    ///     let undone = board.undo_move();
    /// Usage Example:
    ///     if let Some((r, c, p)) = board.undo_move() { ... }
    /// Description:
    ///     Undoes last played move and restores previous bitboard state and hash.
    #[inline(always)]
    pub fn undo_move(&mut self) -> Option<(usize, usize, u8)> {
        if let Some((r, c, player)) = self.history.pop() {
            let idx = r * self.size + c;
            let prev_player = player;

            if player == RED {
                self.red_bb.clear_bit(r, c, self.size);
            } else {
                self.blue_bb.clear_bit(r, c, self.size);
            }

            self.zobrist_hash ^= ZOBRIST_TABLE[idx][player as usize];
            self.zobrist_hash ^= ZOBRIST_PLAYER[self.current_player as usize] ^ ZOBRIST_PLAYER[prev_player as usize];
            self.current_player = prev_player;
            Some((r, c, player))
        } else {
            None
        }
    }

    /// Usage:
    ///     let winner = board.get_winner();
    /// Usage Example:
    ///     if board.get_winner() == BLUE { println!("Blue won!"); }
    /// Description:
    ///     Determines if Red (Top-Bottom) or Blue (Left-Right) has formed a winning path
    ///     using ultra-fast SIMD bitboard flood-fill with bitmask early rejection.
    #[inline(always)]
    pub fn get_winner(&self) -> u8 {
        let top_mask = Bitboard128::row_mask(0, self.size);
        let bottom_mask = Bitboard128::row_mask(self.size - 1, self.size);

        if (self.red_bb.0 & top_mask.0) != 0 && (self.red_bb.0 & bottom_mask.0) != 0 {
            let mut red_front = Bitboard128(self.red_bb.0 & top_mask.0);
            while !red_front.is_empty() {
                if (red_front.0 & bottom_mask.0) != 0 {
                    return RED;
                }
                let expanded = red_front.expand_neighbors(self.size);
                let next_bits = (expanded.0 & self.red_bb.0) & !red_front.0;
                if next_bits == 0 {
                    break;
                }
                red_front.0 |= next_bits;
            }
        }

        let left_mask = Bitboard128::col_mask(0, self.size);
        let right_mask = Bitboard128::col_mask(self.size - 1, self.size);

        if (self.blue_bb.0 & left_mask.0) != 0 && (self.blue_bb.0 & right_mask.0) != 0 {
            let mut blue_front = Bitboard128(self.blue_bb.0 & left_mask.0);
            while !blue_front.is_empty() {
                if (blue_front.0 & right_mask.0) != 0 {
                    return BLUE;
                }
                let expanded = blue_front.expand_neighbors(self.size);
                let next_bits = (expanded.0 & self.blue_bb.0) & !blue_front.0;
                if next_bits == 0 {
                    break;
                }
                blue_front.0 |= next_bits;
            }
        }

        EMPTY
    }

    /// Usage:
    ///     let game_over = board.is_game_over();
    /// Usage Example:
    ///     if board.is_game_over() { ... }
    /// Description:
    ///     Returns true if either Red or Blue has won or no legal moves remain.
    #[inline(always)]
    pub fn is_game_over(&self) -> bool {
        self.get_winner() != EMPTY || (self.red_bb.count_ones() + self.blue_bb.count_ones()) as usize >= self.size * self.size
    }

    /// Usage:
    ///     let moves = board.get_legal_moves();
    /// Usage Example:
    ///     let legal = board.get_legal_moves();
    /// Description:
    ///     Returns list of all unoccupied coordinates (r, c).
    pub fn get_legal_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::with_capacity(self.size * self.size);
        for r in 0..self.size {
            for c in 0..self.size {
                if self.get_cell(r, c) == EMPTY {
                    moves.push((r, c));
                }
            }
        }
        moves
    }
}
