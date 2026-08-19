//! 128-Bit SIMD Bitboard Module for Hex.
//!
//! OOP Description:
//! The `Bitboard128` struct represents a flat hexagonal grid (up to 11x11 = 121 cells)
//! stored directly within a 128-bit unsigned integer register. It implements
//! SIMD bitwise neighbor dilation, border masks, and sub-nanosecond connectivity operations.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bitboard128(pub u128);

impl Bitboard128 {
    /// Usage:
    ///     let empty_bb = Bitboard128::empty();
    /// Usage Example:
    ///     let bb = Bitboard128::empty();
    ///     assert!(bb.is_empty());
    /// Description:
    ///     Returns an empty 128-bit bitboard with all cells cleared.
    #[inline(always)]
    pub fn empty() -> Self {
        Bitboard128(0)
    }

    /// Usage:
    ///     let bb = Bitboard128::from_cell(r, c, size);
    /// Usage Example:
    ///     let bb = Bitboard128::from_cell(5, 5, 11);
    /// Description:
    ///     Constructs a bitboard with a single cell set at row `r` and col `c`.
    #[inline(always)]
    pub fn from_cell(r: usize, c: usize, size: usize) -> Self {
        Bitboard128(1u128 << (r * size + c))
    }

    /// Usage:
    ///     let is_empty = bb.is_empty();
    /// Usage Example:
    ///     if bb.is_empty() { return; }
    /// Description:
    ///     Returns true if no bits are set.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Usage:
    ///     let count = bb.count_ones();
    /// Usage Example:
    ///     let stones = bb.count_ones();
    /// Description:
    ///     Counts number of set bits in bitboard using hardware popcount.
    #[inline(always)]
    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    /// Usage:
    ///     bb.set_bit(r, c, size);
    /// Usage Example:
    ///     bb.set_bit(2, 3, 11);
    /// Description:
    ///     Sets bit corresponding to cell index in place.
    #[inline(always)]
    pub fn set_bit(&mut self, r: usize, c: usize, size: usize) {
        self.0 |= 1u128 << (r * size + c);
    }

    /// Usage:
    ///     bb.clear_bit(r, c, size);
    /// Usage Example:
    ///     bb.clear_bit(2, 3, 11);
    /// Description:
    ///     Clears bit corresponding to cell index in place.
    #[inline(always)]
    pub fn clear_bit(&mut self, r: usize, c: usize, size: usize) {
        self.0 &= !(1u128 << (r * size + c));
    }

    /// Usage:
    ///     let has_stone = bb.has_bit(r, c, size);
    /// Usage Example:
    ///     if bb.has_bit(0, 0, 11) { ... }
    /// Description:
    ///     Returns true if bit corresponding to (r, c) is set.
    #[inline(always)]
    pub fn has_bit(&self, r: usize, c: usize, size: usize) -> bool {
        (self.0 & (1u128 << (r * size + c))) != 0
    }

    /// Usage:
    ///     let bit = bb.get_bit(r, c, size);
    /// Usage Example:
    ///     if bb.get_bit(5, 5, 11) { ... }
    /// Description:
    ///     Alias for has_bit.
    #[inline(always)]
    pub fn get_bit(&self, r: usize, c: usize, size: usize) -> bool {
        self.has_bit(r, c, size)
    }

    /// Usage:
    ///     let neighbors = bb.expand_neighbors(size);
    /// Usage Example:
    ///     let frontier = red_bb.expand_neighbors(11);
    /// Description:
    ///     Dilates bitboard along all 6 hexagonal directions using SIMD bit-shifts:
    ///     (r-1, c), (r-1, c+1), (r, c-1), (r, c+1), (r+1, c-1), (r+1, c).
    #[inline(always)]
    pub fn expand_neighbors(&self, size: usize) -> Self {
        let b = self.0;
        let not_col0 = !Self::col_mask(0, size).0;
        let not_col_last = !Self::col_mask(size - 1, size).0;
        let valid_cells = Self::all_cells_mask(size).0;

        let mut dilated = 0u128;

        // (r, c-1) -> West (valid if not col 0)
        dilated |= (b >> 1) & not_col_last;
        // (r, c+1) -> East (valid if not col last)
        dilated |= (b << 1) & not_col0;

        // (r-1, c) -> North-West
        dilated |= b >> size;
        // (r-1, c+1) -> North-East (valid if not col last)
        dilated |= (b & not_col_last) >> (size - 1);

        // (r+1, c) -> South-East
        dilated |= b << size;
        // (r+1, c-1) -> South-West (valid if not col 0)
        dilated |= (b & not_col0) << (size - 1);

        Bitboard128(dilated & valid_cells)
    }

    /// Usage:
    ///     let mask = Bitboard128::row_mask(r, size);
    /// Usage Example:
    ///     let top_border = Bitboard128::row_mask(0, 11);
    /// Description:
    ///     Constructs bitboard mask for row `r`.
    #[inline(always)]
    pub fn row_mask(r: usize, size: usize) -> Self {
        let row_bits = ((1u128 << size) - 1) << (r * size);
        Bitboard128(row_bits)
    }

    /// Usage:
    ///     let mask = Bitboard128::col_mask(c, size);
    /// Usage Example:
    ///     let left_border = Bitboard128::col_mask(0, 11);
    /// Description:
    ///     Constructs bitboard mask for column `c`.
    #[inline(always)]
    pub fn col_mask(c: usize, size: usize) -> Self {
        let mut mask = 0u128;
        for r in 0..size {
            mask |= 1u128 << (r * size + c);
        }
        Bitboard128(mask)
    }

    /// Usage:
    ///     let mask = Bitboard128::all_cells_mask(size);
    /// Usage Example:
    ///     let full_board = Bitboard128::all_cells_mask(11);
    /// Description:
    ///     Constructs bitboard mask containing all valid cells on board of size `size`.
    #[inline(always)]
    pub fn all_cells_mask(size: usize) -> Self {
        let total = size * size;
        if total >= 128 {
            Bitboard128(!0u128)
        } else {
            Bitboard128((1u128 << total) - 1)
        }
    }
}
