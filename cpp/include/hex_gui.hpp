#pragma once

/**
 * Main Hex GUI Application Controller Module.
 *
 * OOP Description:
 * The `hex::gui::HexGUIApp` class coordinates SDL3 window lifecycle, asynchronous
 * progressive search threads with dynamic seamless depth scaling without resets, 60 FPS live node updates,
 * strategic game plan warden with threat radar, toolbar button events (Undo, Redo, Reset, Clear TT, PGN),
 * modal resize confirmation dialogs, PGN export/import clipboard handling, and hardware-accelerated rendering.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include "hex_engine.hpp"
#include "gui_renderer.hpp"
#include "gui_panel.hpp"
#include "gui_modal.hpp"
#include "gui_top_bar.hpp"
#include "gui_context_menu.hpp"
#include <SDL3/SDL.h>
#include <thread>
#include <atomic>
#include <mutex>
#include <chrono>
#include <string>

namespace hex::gui {

class HexGUIApp {
public:
    SDL_Window* window = nullptr;
    SDL_Renderer* renderer = nullptr;
    HexBoard board;
    HexEngine engine;
    BoardRenderer board_renderer;
    AnalysisPanel analysis_panel;
    ConfirmationModal size_modal;
    ContextMenu context_menu;
    TopBarRenderer top_bar;

    EngineUIStats ui_stats;
    std::mutex stats_mutex;
    std::thread search_thread;
    std::atomic<bool> is_searching{false};
    std::atomic<bool> stop_search{false};
    std::atomic<uint64_t> live_search_nodes{0};
    std::atomic<int> completed_depth{0};
    std::atomic<int> current_searching_depth{1};
    std::atomic<int> target_depth{14};
    std::chrono::steady_clock::time_point search_start_time;

    std::optional<std::pair<int, int>> hover_cell = std::nullopt;
    std::optional<std::pair<int, int>> selected_candidate = std::nullopt;
    MoveTree move_tree;
    std::unordered_map<uint64_t, EngineUIStats> position_cache;
    std::string toast_message = "";
    std::chrono::steady_clock::time_point toast_expiry;
    uint8_t starting_player = BLUE;
    bool running = true;

    /**
     * Usage:
     *     HexGUIApp app(11);
     * Usage Example:
     *     HexGUIApp app(11);
     *     app.run();
     * Description:
     *     Initializes SDL3 window, renderer, and Hex board state.
     */
    explicit HexGUIApp(int board_size = 11) : board(board_size), starting_player(BLUE) {
        board.current_player = starting_player;
        if (!SDL_Init(SDL_INIT_VIDEO)) {
            throw std::runtime_error("SDL_Init failed");
        }
        window = SDL_CreateWindow("Hex Nash Engine - C++ Native Diamond GUI", 1120, 750, SDL_WINDOW_RESIZABLE);
        if (!window) throw std::runtime_error("Failed to create SDL window");

        renderer = SDL_CreateRenderer(window, nullptr);
        if (!renderer) throw std::runtime_error("Failed to create SDL renderer");

        start_fresh_search();
    }

    ~HexGUIApp() {
        stop_async_search();
        if (renderer) SDL_DestroyRenderer(renderer);
        if (window) SDL_DestroyWindow(window);
        SDL_Quit();
    }

    /**
     * Usage:
     *     app.run();
     * Usage Example:
     *     app.run();
     * Description:
     *     Main application loop executing event polling, layout scaling, and 60 FPS rendering.
     */
    void run() {
        while (running) {
            handle_events();
            render();
            SDL_Delay(16);
        }
    }

    /**
     * Usage:
     *     app.change_target_depth(delta);
     * Usage Example:
     *     app.change_target_depth(1);
     * Description:
     *     Dynamically increases or decreases target depth without restarting current computation.
     */
    void change_target_depth(int delta) {
        int old_d = target_depth.load();
        int new_d = std::max(1, old_d + delta);
        if (new_d == old_d) return;

        target_depth.store(new_d);
        if (new_d > old_d) {
            if (!is_searching.load() && completed_depth.load() < new_d) {
                resume_async_search();
            }
        } else if (new_d < old_d) {
            if (is_searching.load() && current_searching_depth.load() > new_d) {
                stop_search.store(true);
            }
        }
    }

    /**
     * Usage:
     *     app.place_human_move(r, c);
     * Usage Example:
     *     app.place_human_move(5, 5);
     * Description:
     *     Places stone for current player, records branch in move tree, and restarts background search.
     */
    void place_human_move(int r, int c) {
        if (board.get_winner() != EMPTY) return;
        uint8_t cur_player = board.current_player;
        if (board.place_move(r, c, cur_player)) {
            move_tree.add_or_select_move(r, c, cur_player);
            start_fresh_search();
        }
    }

    /**
     * Usage: app.undo_move();
     * Example: app.undo_move();
     * Description: Navigates to parent position in move tree (Left Arrow / Cmd+Z / Ctrl+Z).
     */
    void undo_move() {
        if (move_tree.step_backward()) {
            rebuild_board_from_tree();
            show_toast("Undid move (Left Arrow)");
            start_fresh_search();
        }
    }

    /**
     * Usage: app.redo_move();
     * Example: app.redo_move();
     * Description: Navigates to child position in move tree (Right Arrow / Cmd+Shift+Z / Ctrl+Shift+Z).
     */
    void redo_move() {
        if (move_tree.step_forward()) {
            rebuild_board_from_tree();
            show_toast("Redid move (Right Arrow)");
            start_fresh_search();
        }
    }

    /**
     * Usage: app.select_tree_node(node_id);
     * Example: app.select_tree_node(3);
     * Description: Directly navigates board to clicked node in the move tree.
     */
    void select_tree_node(int node_id) {
        if (move_tree.select_node(node_id)) {
            rebuild_board_from_tree();
            show_toast("Jumped to move in tree");
            start_fresh_search();
        }
    }

    /**
     * Usage: app.delete_tree_branch(node_id);
     * Example: app.delete_tree_branch(4);
     * Description: Deletes clicked branch and all its descendants from game tree.
     */
    void delete_tree_branch(int node_id) {
        if (node_id > 0 && node_id < static_cast<int>(move_tree.nodes.size())) {
            std::string alg = move_tree.nodes[node_id].to_algebraic();
            if (move_tree.delete_branch(node_id)) {
                rebuild_board_from_tree();
                show_toast("Removed branch starting at " + alg);
                start_fresh_search();
            }
        }
    }

    /**
     * Usage: app.make_primary_branch(node_id);
     * Example: app.make_primary_branch(4);
     * Description: Promotes variation node (and ancestors) to primary branch in game tree.
     */
    void make_primary_branch(int node_id) {
        if (node_id > 0 && node_id < static_cast<int>(move_tree.nodes.size())) {
            std::string alg = move_tree.nodes[node_id].to_algebraic();
            if (move_tree.make_primary_branch(node_id)) {
                rebuild_board_from_tree();
                show_toast("Promoted branch " + alg + " to Primary");
                start_fresh_search();
            }
        }
    }

    /**
     * Usage: app.reset_game();
     * Example: app.reset_game();
     * Description: Resets board and move tree to empty initial state.
     */
    void reset_game() {
        position_cache.clear();
        engine.clear_cache();
        move_tree.clear();
        board = HexBoard(board.size);
        board.current_player = starting_player;
        completed_depth = 0;
        show_toast("Game Reset to Start");
        start_fresh_search();
    }

    /**
     * Usage: app.request_size_change(new_size);
     * Example: app.request_size_change(9);
     * Description: Requests board resizing; opens confirmation modal if game in progress.
     */
    void request_size_change(int new_size) {
        if (new_size < 3 || new_size > 13 || new_size == board.size) return;
        if (move_tree.is_at_root()) apply_size_change(new_size);
        else size_modal.open(new_size);
    }

    void apply_size_change(int new_size) {
        position_cache.clear();
        engine.clear_cache();
        move_tree.clear();
        board = HexBoard(new_size);
        board.current_player = starting_player;
        completed_depth = 0;
        show_toast("Board resized to " + std::to_string(new_size) + "x" + std::to_string(new_size));
        start_fresh_search();
    }

    /**
     * Usage: app.request_swap_turn();
     * Example: app.request_swap_turn();
     * Description: Requests turn order swap; opens confirmation modal.
     */
    void request_swap_turn() {
        size_modal.open_swap();
    }

    /**
     * Usage: app.swap_turn_order();
     * Example: app.swap_turn_order();
     * Description: Swaps player starting turn order, resets game state, and starts fresh search.
     */
    void swap_turn_order() {
        starting_player = (starting_player == BLUE) ? RED : BLUE;
        position_cache.clear();
        engine.clear_cache();
        move_tree.clear();
        board = HexBoard(board.size);
        board.current_player = starting_player;
        completed_depth = 0;
        show_toast(std::string("Turn order swapped! Starting player: ") + (starting_player == RED ? "RED" : "BLUE"));
        start_fresh_search();
    }

    /**
     * Usage: app.copy_pgn_to_clipboard();
     * Example: app.copy_pgn_to_clipboard();
     * Description: Copies standard PGN representation of active move tree path and starting color to clipboard.
     */
    void copy_pgn_to_clipboard() {
        std::string pgn = move_tree.to_pgn_string(board.size, starting_player);
        SDL_SetClipboardText(pgn.c_str());
        show_toast("Copied PGN to Clipboard!");
    }

    /**
     * Usage: app.import_pgn_from_clipboard();
     * Example: app.import_pgn_from_clipboard();
     * Description: Reads PGN string from clipboard, parses starting color, board size, moves and variations into tree, and replays onto board.
     */
    void import_pgn_from_clipboard() {
        char* clip = SDL_GetClipboardText();
        if (!clip || std::string(clip).empty()) { show_toast("Clipboard is empty!"); return; }
        std::string pgn_str(clip);
        SDL_free(clip);
        uint8_t detected_first_player = starting_player;
        int detected_size = board.size;
        move_tree.load_pgn_tree(pgn_str, board.size, starting_player, &detected_first_player, &detected_size);
        if (move_tree.nodes.size() <= 1) {
            show_toast("No valid moves found in clipboard PGN!");
            return;
        }
        if (detected_size >= 3 && detected_size <= 13 && detected_size != board.size) {
            board = HexBoard(detected_size);
        }
        starting_player = detected_first_player;
        rebuild_board_from_tree();
        show_toast(std::string("Imported Game Tree (") + (starting_player == RED ? "Red" : "Blue") + " went first)!");
        start_fresh_search();
    }

    /**
     * Usage: app.show_toast("Message");
     * Example: app.show_toast("Cache Cleared!");
     * Description: Displays temporary notification banner for 3 seconds.
     */
    void show_toast(const std::string& msg) {
        toast_message = msg;
        toast_expiry = std::chrono::steady_clock::now() + std::chrono::seconds(3);
    }

private:
    void rebuild_board_from_tree() {
        board = HexBoard(board.size);
        board.current_player = starting_player;
        auto history = move_tree.get_path_to_current();
        for (const auto& [r, c] : history) {
            board.place_move(r, c, board.current_player);
        }
    }

    void handle_events() {
        int w = 1120, h = 750;
        SDL_GetWindowSize(window, &w, &h);
        SDL_Event ev;

        while (SDL_PollEvent(&ev)) {
            if (ev.type == SDL_EVENT_QUIT) {
                running = false;
            } else if (ev.type == SDL_EVENT_MOUSE_MOTION && !size_modal.is_visible) {
                hover_cell = board_renderer.pixel_to_hex(ev.motion.x, ev.motion.y, board.size);
                selected_candidate = analysis_panel.get_hovered_candidate(ev.motion.x, ev.motion.y, static_cast<float>(w), ui_stats.top_moves);
            } else if (ev.type == SDL_EVENT_MOUSE_BUTTON_DOWN) {
                if (ev.button.button == SDL_BUTTON_LEFT) {
                    handle_mouse_click(ev.button.x, ev.button.y, static_cast<float>(w), static_cast<float>(h));
                } else if (ev.button.button == SDL_BUTTON_RIGHT) {
                    handle_right_click(ev.button.x, ev.button.y, static_cast<float>(w), static_cast<float>(h));
                }
            } else if (ev.type == SDL_EVENT_KEY_DOWN) {
                handle_key_event(ev);
            }
        }
    }

    void handle_right_click(float mx, float my, float w, float h) {
        if (size_modal.is_visible) return;
        if (auto tok_id = analysis_panel.get_clicked_move_token(mx, my)) {
            bool has_children = move_tree.has_following_nodes(*tok_id);
            context_menu.open(mx, my, *tok_id, has_children, w, h);
        } else {
            context_menu.close();
        }
    }

    void handle_mouse_click(float mx, float my, float w, float h) {
        if (context_menu.is_visible) {
            int target_id = context_menu.target_node_id;
            int act = context_menu.handle_click(mx, my);
            if (act == 1) {
                delete_tree_branch(target_id);
                return;
            } else if (act == 2) {
                make_primary_branch(target_id);
                return;
            }
        }

        if (size_modal.is_visible) {
            int action = size_modal.handle_click(mx, my, w, h);
            if (action == 1) {
                if (size_modal.mode == ConfirmationModal::ModalMode::BOARD_SIZE) {
                    apply_size_change(size_modal.pending_size);
                } else if (size_modal.mode == ConfirmationModal::ModalMode::SWAP_TURN) {
                    swap_turn_order();
                }
            }
            return;
        }
        for (const auto& btn : top_bar.get_buttons()) {
            if (mx >= btn.x && mx <= btn.x + btn.w && my >= btn.y && my <= btn.y + btn.h) {
                if (btn.id == 1) undo_move();
                else if (btn.id == 10) redo_move();
                else if (btn.id == 2) reset_game();
                else if (btn.id == 3) {
                    position_cache.clear();
                    engine.clear_cache();
                    completed_depth = 0;
                    start_fresh_search();
                    show_toast("Transposition Table Cleared!");
                }
                else if (btn.id == 4) copy_pgn_to_clipboard();
                else if (btn.id == 5) import_pgn_from_clipboard();
                else if (btn.id == 6) request_size_change(board.size - 1);
                else if (btn.id == 7) request_size_change(board.size + 1);
                else if (btn.id == 8) change_target_depth(-1);
                else if (btn.id == 9) change_target_depth(1);
                else if (btn.id == 11) request_swap_turn();
                return;
            }
        }
        // Check if move token in analysis panel was clicked
        if (auto tok_id = analysis_panel.get_clicked_move_token(mx, my)) {
            select_tree_node(*tok_id);
            return;
        }
        if (hover_cell.has_value()) place_human_move(hover_cell.value().first, hover_cell.value().second);
    }

    void handle_key_event(const SDL_Event& ev) {
        if (ev.key.key == SDLK_ESCAPE) {
            if (context_menu.is_visible) { context_menu.close(); return; }
            if (size_modal.is_visible) { size_modal.close(); return; }
        }
        if (size_modal.is_visible) return;
        bool is_ctrl = (ev.key.mod & (SDL_KMOD_CTRL | SDL_KMOD_GUI)) != 0;
        bool is_shift = (ev.key.mod & SDL_KMOD_SHIFT) != 0;

        if (ev.key.key == SDLK_LEFT) {
            undo_move();
        } else if (ev.key.key == SDLK_RIGHT) {
            redo_move();
        } else if (is_ctrl && is_shift && ev.key.key == SDLK_Z) {
            redo_move();
        } else if (is_ctrl && !is_shift && ev.key.key == SDLK_Z) {
            undo_move();
        } else if (ev.key.key == SDLK_R) {
            reset_game();
        } else if (ev.key.key == SDLK_TAB) {
            analysis_panel.toggle_collapse();
        } else if (ev.key.key == SDLK_C) {
            position_cache.clear();
            engine.clear_cache();
            completed_depth = 0;
            start_fresh_search();
            show_toast("Transposition Table Cleared!");
        } else if (ev.key.key == SDLK_EQUALS || ev.key.key == SDLK_PLUS) {
            change_target_depth(1);
        } else if (ev.key.key == SDLK_MINUS) {
            change_target_depth(-1);
        }
    }

    void start_fresh_search() {
        stop_async_search();
        uint64_t hash = board.hash();
        int target = target_depth.load();
        int start_d = 1;

        if (board.get_winner() != EMPTY) {
            std::lock_guard<std::mutex> lock(stats_mutex);
            ui_stats.top_moves.clear();
            ui_stats.depth = 0;
            ui_stats.nodes = 0;
            ui_stats.nps = 0;
            ui_stats.elapsed_sec = 0.0;
            ui_stats.eval_score = (board.get_winner() == RED) ? 99999.0f : -99999.0f;
            ui_stats.best_move_str = "Game Over";
            ui_stats.strategic_plan = (board.get_winner() == RED) ? "Red has completed winning path!" : "Blue has completed winning path!";
            ui_stats.threat_level = 3;
            is_searching = false;
            stop_search = false;
            return;
        }

        auto strategy = board.get_strategy(board.current_player);

        {
            std::lock_guard<std::mutex> lock(stats_mutex);
            auto it = position_cache.find(hash);
            if (it != position_cache.end() && !it->second.top_moves.empty()) {
                ui_stats = it->second;
                completed_depth = ui_stats.depth;
                start_d = ui_stats.depth + 1;
            } else {
                // Instantly generate and populate candidate moves for the new position at frame 0
                completed_depth = 0;
                ui_stats.depth = 0;
                ui_stats.nodes = 0;
                ui_stats.nps = 0;
                ui_stats.elapsed_sec = 0.0;
                ui_stats.strategic_plan = strategy.intent;
                ui_stats.threat_level = strategy.threat_level;
                auto fast_candidates = engine.get_initial_candidates(board, board.current_player, 12);
                ui_stats.top_moves = fast_candidates;
                if (!fast_candidates.empty()) {
                    ui_stats.eval_score = fast_candidates[0].score;
                    ui_stats.best_move_str = std::string(1, static_cast<char>('A' + fast_candidates[0].c)) + std::to_string(fast_candidates[0].r + 1);
                } else {
                    ui_stats.eval_score = 0.0f;
                    ui_stats.best_move_str = "-";
                }
                position_cache[hash] = ui_stats;
                start_d = 1;
            }
        }

        live_search_nodes = 0;
        search_start_time = std::chrono::steady_clock::now();
        if (start_d <= target) {
            is_searching = true;
            stop_search = false;
            launch_search_worker(start_d, target);
        } else {
            is_searching = false;
            stop_search = false;
        }
    }

    void resume_async_search() {
        if (is_searching.load()) return;
        stop_async_search();
        int next_d = completed_depth.load() + 1;
        int max_d = target_depth.load();
        if (next_d <= max_d) {
            is_searching = true;
            stop_search = false;
            launch_search_worker(next_d, max_d);
        }
    }

    void backpropagate_eval_to_parent(const std::vector<std::pair<int, int>>& path, int d, float child_eval_score) {
        if (path.empty()) return;
        auto last_move = path.back();
        HexBoard parent_board(board.size);
        parent_board.current_player = starting_player;
        for (size_t i = 0; i + 1 < path.size(); i++) {
            parent_board.place_move(path[i].first, path[i].second, parent_board.current_player);
        }

        uint64_t p_hash = parent_board.hash();
        auto it = position_cache.find(p_hash);
        if (it != position_cache.end() && !it->second.top_moves.empty()) {
            auto& parent_stats = it->second;
            bool found = false;
            for (auto& entry : parent_stats.top_moves) {
                if (entry.r == last_move.first && entry.c == last_move.second) {
                    entry.score = child_eval_score;
                    entry.depth = std::max(entry.depth, d + 1);
                    found = true;
                    break;
                }
            }
            if (found) {
                uint8_t p_player = parent_board.current_player;
                std::sort(parent_stats.top_moves.begin(), parent_stats.top_moves.end(),
                    [p_player](const TopMove& a, const TopMove& b) {
                        return (p_player == RED) ? (a.score > b.score) : (a.score < b.score);
                    });
                for (size_t rk = 0; rk < parent_stats.top_moves.size(); rk++) {
                    parent_stats.top_moves[rk].rank = static_cast<int>(rk + 1);
                }
                if (!parent_stats.top_moves.empty()) {
                    const auto& best_top = parent_stats.top_moves[0];
                    parent_stats.eval_score = best_top.score;
                    parent_stats.best_move_str = std::string(1, static_cast<char>('A' + best_top.c)) + std::to_string(best_top.r + 1);
                }
            }
        }

        // Recursively backpropagate up the ancestor tree
        if (path.size() > 1) {
            std::vector<std::pair<int, int>> parent_path(path.begin(), path.end() - 1);
            backpropagate_eval_to_parent(parent_path, d, child_eval_score);
        }
    }

    void launch_search_worker(int start_d, int max_d) {
        HexBoard search_board = board;
        uint64_t search_board_hash = search_board.hash();
        uint8_t player = board.current_player;
        auto strategy = search_board.get_strategy(player);
        auto current_path = move_tree.get_path_to_current();

        search_thread = std::thread([this, search_board, search_board_hash, player, start_d, max_d, strategy, current_path]() {
            auto on_live_candidates = [](void* user_data, const CTopMoveEntry* entries, int count, int b_r, int b_c, float b_s) {
                auto* self = static_cast<HexGUIApp*>(user_data);
                if (!self || self->stop_search.load()) return;
                std::string b_str = "-";
                if (b_r >= 0 && b_c >= 0) {
                    b_str = std::string(1, static_cast<char>('A' + b_c)) + std::to_string(b_r + 1);
                }
                std::vector<TopMove> live_moves;
                live_moves.reserve(count);
                for (int i = 0; i < count; i++) {
                    live_moves.push_back({
                        entries[i].rank,
                        entries[i].r,
                        entries[i].c,
                        entries[i].score,
                        entries[i].depth
                    });
                }
                {
                    std::lock_guard<std::mutex> lock(self->stats_mutex);
                    self->ui_stats.top_moves = live_moves;
                    self->ui_stats.best_move_str = b_str;
                    self->ui_stats.eval_score = b_s;
                    self->position_cache[self->board.hash()] = self->ui_stats;
                }
            };

            for (int d = start_d; d <= max_d && !stop_search.load(); d++) {
                current_searching_depth = d;
                auto res = engine.search_depth_step_streaming(
                    search_board,
                    player,
                    d,
                    &stop_search,
                    &live_search_nodes,
                    on_live_candidates,
                    this
                );
                if (stop_search.load()) break;

                completed_depth.store(d);
                auto t1 = std::chrono::steady_clock::now();
                double elapsed = std::chrono::duration<double>(t1 - search_start_time).count();
                uint64_t nodes = live_search_nodes.load();
                uint64_t nps = static_cast<uint64_t>(nodes / std::max(0.0001, elapsed));

                std::string best_str = "-";
                if (res.best_move.has_value()) {
                    auto [r, c] = res.best_move.value();
                    best_str = std::string(1, static_cast<char>('A' + c)) + std::to_string(r + 1);
                }

                {
                    std::lock_guard<std::mutex> lock(stats_mutex);
                    if (board.hash() == search_board_hash) {
                        ui_stats.nodes = nodes;
                        ui_stats.nps = nps;
                        ui_stats.elapsed_sec = elapsed;
                        ui_stats.depth = d;
                        ui_stats.eval_score = res.score;
                        ui_stats.best_move_str = best_str;
                        ui_stats.is_searching = (d < max_d);
                        ui_stats.top_moves = res.top_moves;
                        ui_stats.strategic_plan = strategy.intent;
                        ui_stats.threat_level = strategy.threat_level;
                    }
                    EngineUIStats cached_stat;
                    cached_stat.nodes = nodes;
                    cached_stat.nps = nps;
                    cached_stat.elapsed_sec = elapsed;
                    cached_stat.depth = d;
                    cached_stat.eval_score = res.score;
                    cached_stat.best_move_str = best_str;
                    cached_stat.is_searching = (d < max_d);
                    cached_stat.top_moves = res.top_moves;
                    cached_stat.strategic_plan = strategy.intent;
                    cached_stat.threat_level = strategy.threat_level;
                    position_cache[search_board_hash] = cached_stat;
                    backpropagate_eval_to_parent(current_path, d, res.score);
                }
                if (std::abs(res.score) >= 90000.0f) break;
            }
            is_searching = false;
        });
    }

    void stop_async_search() {
        stop_search = true;
        if (search_thread.joinable()) search_thread.join();
        is_searching = false;
    }

    void render() {
        int w = 1120, h = 750;
        SDL_GetWindowSize(window, &w, &h);

        SDL_SetRenderDrawColor(renderer, 245, 247, 250, 255);
        SDL_RenderClear(renderer);

        top_bar.render(renderer, static_cast<float>(w), board.size, target_depth.load(), board.current_player, board.get_winner());

        EngineUIStats current_stats;
        {
            std::lock_guard<std::mutex> lock(stats_mutex);
            current_stats = ui_stats;
            current_stats.is_searching = is_searching.load();
        }

        if (current_stats.is_searching) {
            uint64_t live_n = live_search_nodes.load();
            auto now = std::chrono::steady_clock::now();
            double live_elapsed = std::chrono::duration<double>(now - search_start_time).count();
            current_stats.nodes = live_n;
            current_stats.elapsed_sec = live_elapsed;
            current_stats.nps = static_cast<uint64_t>(live_n / std::max(0.0001, live_elapsed));
            current_stats.depth = current_searching_depth.load();
        }

        board_renderer.update_layout(static_cast<float>(w), static_cast<float>(h), board.size, analysis_panel.is_collapsed);
        analysis_panel.draw_eval_bar(renderer, 20.0f, 70.0f, 28.0f, static_cast<float>(h) - 140.0f, current_stats.eval_score);
        auto book_moves = engine.get_book_moves(board, board.current_player);
        board_renderer.draw_board(renderer, board, hover_cell, current_stats.top_moves, selected_candidate, book_moves);

        float mx = -1.0f, my = -1.0f;
        SDL_GetMouseState(&mx, &my);
        analysis_panel.draw_panel(renderer, static_cast<float>(w), static_cast<float>(h), current_stats, move_tree, mx, my);

        if (!toast_message.empty() && std::chrono::steady_clock::now() < toast_expiry) {
            float banner_w = 340.0f;
            float bx = (static_cast<float>(w) - banner_w) / 2.0f;
            float by = static_cast<float>(h) - 48.0f;
            SDL_SetRenderDrawColor(renderer, 20, 25, 35, 240);
            SDL_FRect toast_rect{bx, by, banner_w, 36.0f};
            SDL_RenderFillRect(renderer, &toast_rect);
            SDL_SetRenderDrawColor(renderer, 76, 175, 80, 255);
            SDL_RenderRect(renderer, &toast_rect);
            SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);
            SDL_RenderDebugText(renderer, bx + 16.0f, by + 10.0f, toast_message.c_str());
        }

        size_modal.render(renderer, static_cast<float>(w), static_cast<float>(h));
        context_menu.render(renderer, mx, my);
        SDL_RenderPresent(renderer);
    }
};

} // namespace hex::gui
