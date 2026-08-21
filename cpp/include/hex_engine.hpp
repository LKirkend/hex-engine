#pragma once

/**
 * Modern C++20 Hex Nash Engine Header Wrapper.
 *
 * OOP Description:
 * The `hex::HexEngine` and `hex::HexBoard` classes encapsulate the high-speed
 * SIMD Bitboard Hex engine through type-safe C++20 abstractions, providing
 * zero-cost RAII resource management, move semantics, and fast minimax search.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include <cstdint>
#include <vector>
#include <string>
#include <optional>
#include <memory>
#include <stdexcept>
#include <iostream>
#include <atomic>

extern "C" {
    struct CTopMoveEntry {
        int32_t rank;
        int32_t r;
        int32_t c;
        float score;
        int32_t depth;
    };

    void* hex_engine_create();
    void hex_engine_free(void* ptr);
    float hex_engine_evaluate(const uint8_t* grid_ptr, int32_t size);
    int32_t hex_engine_get_winner(const uint8_t* grid_ptr, int32_t size);
    int32_t hex_engine_search_with_cancel(
        void* engine_ptr,
        const uint8_t* grid_ptr,
        int32_t size,
        int32_t player,
        int32_t depth,
        const std::atomic<bool>* cancel_flag_ptr,
        int32_t* out_r,
        int32_t* out_c,
        float* out_score,
        uint64_t* out_nodes,
        CTopMoveEntry* out_top_moves,
        int32_t max_top,
        int32_t* out_num_top
    );
    typedef void (*LiveCandidateCallback)(
        void* user_data,
        const CTopMoveEntry* moves,
        int num_moves,
        int best_r,
        int best_c,
        float best_score
    );

    int32_t hex_engine_search_step(
        void* engine_ptr,
        const uint8_t* grid_ptr,
        int32_t size,
        int32_t player,
        int32_t depth,
        const std::atomic<bool>* cancel_flag_ptr,
        const std::atomic<uint64_t>* live_nodes_ptr,
        int32_t* out_r,
        int32_t* out_c,
        float* out_score,
        uint64_t* out_nodes,
        CTopMoveEntry* out_top_moves,
        int32_t max_top,
        int32_t* out_num_top
    );
    int32_t hex_engine_search_step_streaming(
        void* engine_ptr,
        const uint8_t* grid_ptr,
        int32_t size,
        int32_t player,
        int32_t depth,
        const std::atomic<bool>* cancel_flag_ptr,
        const std::atomic<uint64_t>* live_nodes_ptr,
        LiveCandidateCallback live_callback,
        void* user_data,
        int32_t* out_r,
        int32_t* out_c,
        float* out_score,
        uint64_t* out_nodes,
        CTopMoveEntry* out_top_moves,
        int32_t max_top,
        int32_t* out_num_top
    );
    int32_t hex_engine_get_strategy(
        const uint8_t* grid_ptr,
        int32_t size,
        int32_t player,
        char* out_intent_buf,
        int32_t buf_len,
        int32_t* out_threat
    );
    void hex_engine_clear_tt(void* engine_ptr);
    int32_t hex_engine_get_book_moves(
        const uint8_t* grid_ptr,
        int32_t size,
        int32_t player,
        int32_t* out_r,
        int32_t* out_c,
        int32_t max_moves,
        int32_t* out_count
    );
    int32_t hex_engine_get_initial_candidates(
        void* engine_ptr,
        const uint8_t* grid_ptr,
        int32_t size,
        int32_t player,
        CTopMoveEntry* out_top_moves,
        int32_t max_top,
        int32_t* out_num_top
    );

    void* nic_engine_create(int32_t depth);
    void nic_engine_destroy(void* ptr);
    int32_t nic_engine_select_move(
        void* nic_ptr,
        const uint8_t* grid_ptr,
        int32_t size,
        int32_t player,
        int32_t* out_r,
        int32_t* out_c,
        float* out_score
    );
}

namespace hex {

constexpr uint8_t EMPTY = 0;
constexpr uint8_t RED = 1;
constexpr uint8_t BLUE = 2;

struct TopMove {
    int rank;
    int r;
    int c;
    float score;
    int depth;
};

struct SearchResult {
    std::optional<std::pair<int, int>> best_move;
    float score;
    uint64_t nodes;
    std::vector<TopMove> top_moves;
};

struct StrategyAdvice {
    std::string intent;
    int threat_level;
};

class HexBoard {
public:
    int size;
    std::vector<uint8_t> grid;
    uint8_t current_player;

    /**
     * Usage:
     *     hex::HexBoard board(11);
     * Usage Example:
     *     hex::HexBoard board(11);
     *     board.place_move(5, 5, hex::BLUE);
     * Description:
     *     Initializes flat hexagonal grid of dimension size x size.
     */
    explicit HexBoard(int s = 11) : size(s), grid(s * s, EMPTY), current_player(BLUE) {}

    /**
     * Usage:
     *     bool ok = board.place_move(r, c, player);
     * Usage Example:
     *     board.place_move(0, 0, hex::RED);
     * Description:
     *     Places a piece on the board if the target cell is unoccupied.
     */
    bool place_move(int r, int c, uint8_t player) {
        if (r < 0 || r >= size || c < 0 || c >= size) return false;
        int idx = r * size + c;
        if (grid[idx] != EMPTY) return false;
        grid[idx] = player;
        current_player = (player == RED) ? BLUE : RED;
        return true;
    }

    /**
     * Usage:
     *     uint8_t winner = board.get_winner();
     * Usage Example:
     *     if (board.get_winner() == hex::RED) { ... }
     * Description:
     *     Determines whether Red or Blue has connected opposing borders.
     */
    uint8_t get_winner() const {
        return static_cast<uint8_t>(hex_engine_get_winner(grid.data(), size));
    }

    /**
     * Usage:
     *     float score = board.evaluate();
     * Usage Example:
     *     float eval_score = board.evaluate();
     * Description:
     *     Calculates static position evaluation score from Red's perspective.
     */
    float evaluate() const {
        return hex_engine_evaluate(grid.data(), size);
    }

    /**
     * Usage:
     *     auto plan = board.get_strategy(player);
     * Usage Example:
     *     auto plan = board.get_strategy(hex::RED);
     * Description:
     *     Returns high-level tactical game plan and threat radar assessment.
     */
    StrategyAdvice get_strategy(uint8_t player) const {
        char buf[256] = {0};
        int32_t threat = 1;
        hex_engine_get_strategy(grid.data(), size, player, buf, 256, &threat);
        return {std::string(buf), threat};
    }

    /**
     * Usage:
     *     uint64_t h = board.hash();
     * Usage Example:
     *     uint64_t h = board.hash();
     * Description:
     *     Calculates 64-bit FNV-1a hash over board occupancy and current player.
     */
    uint64_t hash() const {
        uint64_t h = 14695981039346656037ULL;
        for (uint8_t cell : grid) {
            h ^= cell;
            h *= 1099511628211ULL;
        }
        h ^= current_player;
        h *= 1099511628211ULL;
        return h;
    }
};

class HexEngine {
private:
    void* handle = nullptr;

public:
    /**
     * Usage:
     *     hex::HexEngine engine;
     * Usage Example:
     *     hex::HexEngine engine;
     *     auto res = engine.search(board, hex::RED, 6);
     * Description:
     *     Creates and manages lifetime of the underlying Rust SearchEngine instance.
     */
    HexEngine() {
        handle = hex_engine_create();
        if (!handle) {
            throw std::runtime_error("Failed to initialize Rust Hex Engine core");
        }
    }

    ~HexEngine() {
        if (handle) {
            hex_engine_free(handle);
            handle = nullptr;
        }
    }

    HexEngine(const HexEngine&) = delete;
    HexEngine& operator=(const HexEngine&) = delete;

    HexEngine(HexEngine&& other) noexcept : handle(other.handle) {
        other.handle = nullptr;
    }

    HexEngine& operator=(HexEngine&& other) noexcept {
        if (this != &other) {
            if (handle) hex_engine_free(handle);
            handle = other.handle;
            other.handle = nullptr;
        }
        return *this;
    }

    /**
     * Usage:
     *     auto res = engine.search(board, player, depth, cancel_flag);
     * Usage Example:
     *     auto res = engine.search(board, hex::BLUE, 8);
     * Description:
     *     Executes iterative deepening PVS search for the specified player.
     */
    SearchResult search(
        const HexBoard& board,
        uint8_t player,
        int depth,
        const std::atomic<bool>* cancel_flag = nullptr
    ) {
        int32_t best_r = -1;
        int32_t best_c = -1;
        float best_score = 0.0f;
        uint64_t total_nodes = 0;
        CTopMoveEntry top_entries[12];
        int32_t num_top = 0;

        int status = hex_engine_search_with_cancel(
            handle,
            board.grid.data(),
            board.size,
            player,
            depth,
            cancel_flag,
            &best_r,
            &best_c,
            &best_score,
            &total_nodes,
            top_entries,
            12,
            &num_top
        );

        SearchResult result;
        if (status && best_r >= 0 && best_c >= 0) {
            result.best_move = std::make_pair(best_r, best_c);
        } else {
            result.best_move = std::nullopt;
        }
        result.score = best_score;
        result.nodes = total_nodes;

        for (int i = 0; i < num_top; i++) {
            result.top_moves.push_back({
                top_entries[i].rank,
                top_entries[i].r,
                top_entries[i].c,
                top_entries[i].score,
                top_entries[i].depth
            });
        }

        return result;
    }

    /**
     * Usage:
     *     auto res = engine.search_depth_step(board, player, depth, cancel_flag, live_nodes);
     * Usage Example:
     *     auto res = engine.search_depth_step(board, hex::RED, 4, &stop, &live_nodes);
     * Description:
     *     Executes single depth search step with live atomic node reporting.
     */
    SearchResult search_depth_step(
        const HexBoard& board,
        uint8_t player,
        int depth,
        const std::atomic<bool>* cancel_flag = nullptr,
        const std::atomic<uint64_t>* live_nodes = nullptr
    ) {
        int32_t best_r = -1;
        int32_t best_c = -1;
        float best_score = 0.0f;
        uint64_t total_nodes = 0;
        CTopMoveEntry top_entries[12];
        int32_t num_top = 0;

        int status = hex_engine_search_step(
            handle,
            board.grid.data(),
            board.size,
            player,
            depth,
            cancel_flag,
            live_nodes,
            &best_r,
            &best_c,
            &best_score,
            &total_nodes,
            top_entries,
            12,
            &num_top
        );

        SearchResult result;
        if (status && best_r >= 0 && best_c >= 0) {
            result.best_move = std::make_pair(best_r, best_c);
        } else {
            result.best_move = std::nullopt;
        }
        result.score = best_score;
        result.nodes = total_nodes;

        for (int i = 0; i < num_top; i++) {
            result.top_moves.push_back({
                top_entries[i].rank,
                top_entries[i].r,
                top_entries[i].c,
                top_entries[i].score,
                top_entries[i].depth
            });
        }

        return result;
    }

    /**
     * Usage:
     *     auto res = engine.search_depth_step_streaming(board, player, d, cancel, nodes, cb, user_data);
     * Usage Example:
     *     auto res = engine.search_depth_step_streaming(board, hex::RED, 6, &cancel, &nodes, cb, this);
     * Description:
     *     Executes a single depth step with real-time candidate move streaming callback.
     */
    SearchResult search_depth_step_streaming(
        const HexBoard& board,
        uint8_t player,
        int depth,
        const std::atomic<bool>* cancel_flag = nullptr,
        const std::atomic<uint64_t>* live_nodes = nullptr,
        LiveCandidateCallback live_callback = nullptr,
        void* user_data = nullptr
    ) {
        int32_t best_r = -1;
        int32_t best_c = -1;
        float best_score = 0.0f;
        uint64_t total_nodes = 0;
        CTopMoveEntry top_entries[12];
        int32_t num_top = 0;

        int status = hex_engine_search_step_streaming(
            handle,
            board.grid.data(),
            board.size,
            player,
            depth,
            cancel_flag,
            live_nodes,
            live_callback,
            user_data,
            &best_r,
            &best_c,
            &best_score,
            &total_nodes,
            top_entries,
            12,
            &num_top
        );

        SearchResult result;
        if (status && best_r >= 0 && best_c >= 0) {
            result.best_move = std::make_pair(best_r, best_c);
        } else {
            result.best_move = std::nullopt;
        }
        result.score = best_score;
        result.nodes = total_nodes;

        for (int i = 0; i < num_top; i++) {
            result.top_moves.push_back({
                top_entries[i].rank,
                top_entries[i].r,
                top_entries[i].c,
                top_entries[i].score,
                top_entries[i].depth
            });
        }

        return result;
    }

    /**
     * Usage:
     *     engine.clear_cache();
     * Usage Example:
     *     engine.clear_cache();
     * Description:
     *     Clears all transposition table entries.
     */
    void clear_cache() {
        hex_engine_clear_tt(handle);
    }

    /**
     * Usage:
     *     auto book_moves = engine.get_book_moves(board, player);
     * Usage Example:
     *     auto book_moves = engine.get_book_moves(board, hex::BLUE);
     * Description:
     *     Retrieves all active game-theoretic opening book moves for current position.
     */
    std::vector<std::pair<int, int>> get_book_moves(const HexBoard& board, uint8_t player) {
        std::vector<int32_t> r_buf(16);
        std::vector<int32_t> c_buf(16);
        int32_t count = 0;
        hex_engine_get_book_moves(board.grid.data(), board.size, player, r_buf.data(), c_buf.data(), 16, &count);
        std::vector<std::pair<int, int>> res;
        for (int i = 0; i < count; i++) {
            res.push_back({r_buf[i], c_buf[i]});
        }
        return res;
    }

    /**
     * Usage:
     *     auto candidates = engine.get_initial_candidates(board, player, 12);
     * Usage Example:
     *     auto candidates = engine.get_initial_candidates(board, hex::BLUE, 12);
     * Description:
     *     Instantly returns candidate move leaderboard for a position using TT lookups and fast heuristic.
     */
    std::vector<TopMove> get_initial_candidates(const HexBoard& board, uint8_t player, int max_top = 12) {
        std::vector<TopMove> results;
        if (!handle) return results;

        std::vector<CTopMoveEntry> c_entries(max_top);
        int32_t num_top = 0;
        int status = hex_engine_get_initial_candidates(
            handle,
            board.grid.data(),
            board.size,
            player,
            c_entries.data(),
            max_top,
            &num_top
        );

        if (status != 0 && num_top > 0) {
            results.reserve(num_top);
            for (int i = 0; i < num_top; i++) {
                results.push_back({
                    c_entries[i].rank,
                    c_entries[i].r,
                    c_entries[i].c,
                    c_entries[i].score,
                    c_entries[i].depth
                });
            }
        }
        return results;
    }
};

/**
 * Nintendo Impossible Computer (NIC) C++ RAII Engine Wrapper.
 */
class NicEngine {
public:
    void* handle = nullptr;

    explicit NicEngine(int depth = 3) {
        handle = nic_engine_create(depth);
    }

    ~NicEngine() {
        if (handle) {
            nic_engine_destroy(handle);
            handle = nullptr;
        }
    }

    NicEngine(const NicEngine&) = delete;
    NicEngine& operator=(const NicEngine&) = delete;

    NicEngine(NicEngine&& other) noexcept : handle(other.handle) {
        other.handle = nullptr;
    }

    NicEngine& operator=(NicEngine&& other) noexcept {
        if (this != &other) {
            if (handle) nic_engine_destroy(handle);
            handle = other.handle;
            other.handle = nullptr;
        }
        return *this;
    }

    /**
     * Usage:
     *     auto [move, score] = nic.select_move(board, player);
     * Description:
     *     Executes move selection using Nintendo's Anshelevich VC / conductance gradient AI.
     */
    std::pair<std::optional<std::pair<int, int>>, float> select_move(const HexBoard& board, uint8_t player) {
        if (!handle) return {std::nullopt, 0.0f};
        int32_t out_r = -1;
        int32_t out_c = -1;
        float out_score = 0.0f;

        int status = nic_engine_select_move(
            handle,
            board.grid.data(),
            board.size,
            player,
            &out_r,
            &out_c,
            &out_score
        );

        if (status != 0 && out_r >= 0 && out_c >= 0) {
            return {std::make_pair(out_r, out_c), out_score};
        }
        return {std::nullopt, 0.0f};
    }
};

} // namespace hex
