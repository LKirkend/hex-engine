//! Lock-Free Atomic Transposition Table Module.
//!
//! OOP Description:
//! The `TranspositionTable` struct provides a thread-safe, lock-free memoization
//! cache for multi-threaded Lazy SMP minimax search. It packs hash keys, depth,
//! evaluation bounds, and optimal moves into atomic integers.
//! Default Author: Logan Kirkendall, Logan@LKAud.io

use std::sync::atomic::{AtomicU64, Ordering};

pub const EXACT: u8 = 0;
pub const LOWERBOUND: u8 = 1;
pub const UPPERBOUND: u8 = 2;

const TT_ENTRIES: usize = 4194304; // 4M entries (~64 MB)

#[derive(Default)]
struct TTEntry {
    key: AtomicU64,
    data: AtomicU64, // Packed: score (f32 as u32), depth (u8), flag (u8), r (u8), c (u8)
}

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
}

impl TranspositionTable {
    /// Usage:
    ///     let tt = TranspositionTable::new();
    /// Usage Example:
    ///     let tt = TranspositionTable::new();
    /// Description:
    ///     Initializes a lock-free transposition table with 1,048,576 atomic entry slots.
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(TT_ENTRIES);
        for _ in 0..TT_ENTRIES {
            entries.push(TTEntry::default());
        }
        TranspositionTable { entries }
    }

    /// Usage:
    ///     tt.store(hash, depth, score, flag, best_move);
    /// Usage Example:
    ///     tt.store(0x12345, 6, 23.5, EXACT, Some((5, 5)));
    /// Description:
    ///     Atomically stores search evaluation bounds and best move into table.
    pub fn store(&self, hash: u64, depth: u8, score: f32, flag: u8, best_move: Option<(usize, usize)>) {
        let idx = (hash as usize) & (TT_ENTRIES - 1);
        let entry = &self.entries[idx];

        let (r, c) = best_move.unwrap_or((255, 255));
        let score_bits = score.to_bits();

        let packed_data = (score_bits as u64)
            | ((depth as u64) << 32)
            | ((flag as u64) << 40)
            | ((r as u64) << 48)
            | ((c as u64) << 56);

        entry.data.store(packed_data, Ordering::Relaxed);
        entry.key.store(hash, Ordering::Release);
    }

    /// Usage:
    ///     let res = tt.lookup(hash, depth, alpha, beta);
    /// Usage Example:
    ///     if let Some((score, mv)) = tt.lookup(hash, 4, -100.0, 100.0) { ... }
    /// Description:
    ///     Atomically looks up cached position and returns score and move if bounds match.
    pub fn lookup(&self, hash: u64, depth: u8, alpha: f32, beta: f32) -> Option<(f32, Option<(usize, usize)>)> {
        let idx = (hash as usize) & (TT_ENTRIES - 1);
        let entry = &self.entries[idx];

        if entry.key.load(Ordering::Acquire) != hash {
            return None;
        }

        let data = entry.data.load(Ordering::Relaxed);
        let score_bits = (data & 0xFFFFFFFF) as u32;
        let score = f32::from_bits(score_bits);
        let cached_depth = ((data >> 32) & 0xFF) as u8;
        let flag = ((data >> 40) & 0xFF) as u8;
        let r = ((data >> 48) & 0xFF) as usize;
        let c = ((data >> 56) & 0xFF) as usize;

        let best_move = if r < 255 && c < 255 { Some((r, c)) } else { None };

        if cached_depth >= depth {
            if flag == EXACT {
                return Some((score, best_move));
            } else if flag == LOWERBOUND && score >= beta {
                return Some((score, best_move));
            } else if flag == UPPERBOUND && score <= alpha {
                return Some((score, best_move));
            }
        }

        None
    }

    /// Usage:
    ///     if let Some((score, d)) = tt.get_entry_score(hash) { ... }
    /// Usage Example:
    ///     let (score, depth) = tt.get_entry_score(child_hash).unwrap();
    /// Description:
    ///     Retrieves cached score and depth for hash without alpha/beta bound filtering.
    pub fn get_entry_score(&self, hash: u64) -> Option<(f32, u8)> {
        let idx = (hash as usize) & (TT_ENTRIES - 1);
        let entry = &self.entries[idx];

        if entry.key.load(Ordering::Acquire) != hash {
            return None;
        }

        let data = entry.data.load(Ordering::Relaxed);
        let score_bits = (data & 0xFFFFFFFF) as u32;
        let score = f32::from_bits(score_bits);
        let cached_depth = ((data >> 32) & 0xFF) as u8;
        Some((score, cached_depth))
    }

    /// Usage:
    ///     let mv = tt.get_best_move(hash);
    /// Usage Example:
    ///     let cached_pv = tt.get_best_move(hash);
    /// Description:
    ///     Extracts cached best move without depth pruning for move ordering priority.
    pub fn get_best_move(&self, hash: u64) -> Option<(usize, usize)> {
        let idx = (hash as usize) & (TT_ENTRIES - 1);
        let entry = &self.entries[idx];

        if entry.key.load(Ordering::Acquire) != hash {
            return None;
        }

        let data = entry.data.load(Ordering::Relaxed);
        let r = ((data >> 48) & 0xFF) as usize;
        let c = ((data >> 56) & 0xFF) as usize;

        if r < 255 && c < 255 {
            Some((r, c))
        } else {
            None
        }
    }

    /// Usage:
    ///     tt.clear();
    /// Usage Example:
    ///     tt.clear();
    /// Description:
    ///     Clears all entries in the transposition table.
    pub fn clear(&self) {
        for entry in &self.entries {
            entry.key.store(0, Ordering::Relaxed);
            entry.data.store(0, Ordering::Relaxed);
        }
    }
}
