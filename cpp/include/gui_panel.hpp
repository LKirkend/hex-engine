#pragma once

/**
 * Hex GUI Analysis Panel, Evaluation Meter, and PGN Move History Module.
 *
 * OOP Description:
 * The `hex::gui::AnalysisPanel` class renders the collapsible candidate move leaderboard,
 * chess-style evaluation meter, strategic game plan warden with threat radar,
 * live PGN move history display, engine search statistics, and handles candidate hover hit-testing.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include "hex_engine.hpp"
#include "gui_move_tree.hpp"
#include <SDL3/SDL.h>
#include <vector>
#include <string>
#include <iomanip>
#include <sstream>
#include <optional>
#include <cctype>

namespace hex::gui {

struct EngineUIStats {
    uint64_t nodes = 0;
    uint64_t nps = 0;
    double elapsed_sec = 0.0;
    int depth = 8;
    float eval_score = 0.0f;
    std::string best_move_str = "-";
    bool is_searching = false;
    std::string strategic_plan = "Developing central 2-bridge network.";
    int threat_level = 1;
    std::vector<TopMove> top_moves;
};

/**
 * Usage:
 *     std::string pgn = format_pgn_string(history, 11, hex::BLUE);
 * Usage Example:
 *     auto pgn = format_pgn_string(history, 11, hex::RED);
 * Description:
 *     Formats move history into standard algebraic notation PGN string (e.g. 1. f6 g6 2. g5 d7),
 *     including which player color went first in the [First "Color"] header.
 */
inline std::string format_pgn_string(const std::vector<std::pair<int, int>>& history, int size, uint8_t starting_player = 2) {
    std::stringstream ss;
    ss << "[Game \"Hex\"]\n[Size \"" << size << "x" << size << "\"]\n";
    ss << "[First \"" << (starting_player == 1 ? "Red" : "Blue") << "\"]\n\n";

    for (size_t i = 0; i < history.size(); i++) {
        if (i % 2 == 0) {
            ss << (i / 2 + 1) << ". ";
        }
        char col_char = static_cast<char>('a' + history[i].second);
        ss << col_char << (history[i].first + 1) << " ";
    }
    return ss.str();
}

/**
 * Usage:
 *     auto moves = parse_pgn_string(pgn_text, 11, &first_player, &size);
 * Usage Example:
 *     auto moves = parse_pgn_string("1. f6 g6 2. g5 d7", 11);
 * Description:
 *     Parses algebraic move coordinates from PGN string text, returning valid (r, c) moves,
 *     and extracting which color went first and board size if present.
 */
inline std::vector<std::pair<int, int>> parse_pgn_string(
    const std::string& pgn_text,
    int size,
    uint8_t* out_first_player = nullptr,
    int* out_size = nullptr
) {
    std::vector<std::pair<int, int>> moves;
    size_t i = 0;
    uint8_t first_player = 2;
    int parsed_size = size;

    while (i < pgn_text.size()) {
        if (pgn_text[i] == '[') {
            size_t tag_start = i;
            while (i < pgn_text.size() && pgn_text[i] != ']') i++;
            if (i < pgn_text.size()) i++;
            std::string tag_str = pgn_text.substr(tag_start, i - tag_start);
            std::string key, val;
            if (parse_pgn_tag(tag_str, key, val)) {
                process_pgn_tag(key, val, first_player, parsed_size);
            }
            continue;
        }

        if (std::isdigit(static_cast<unsigned char>(pgn_text[i]))) {
            size_t num_start = i;
            while (i < pgn_text.size() && std::isdigit(static_cast<unsigned char>(pgn_text[i]))) i++;
            if (i < pgn_text.size() && pgn_text[i] == '.') {
                i++;
                continue;
            } else {
                i = num_start;
            }
        }

        if (std::isalpha(static_cast<unsigned char>(pgn_text[i]))) {
            char col_char = static_cast<char>(std::tolower(static_cast<unsigned char>(pgn_text[i])));
            int c = col_char - 'a';
            i++;

            std::string row_str;
            while (i < pgn_text.size() && std::isdigit(static_cast<unsigned char>(pgn_text[i]))) {
                row_str += pgn_text[i];
                i++;
            }

            if (!row_str.empty()) {
                int r = std::stoi(row_str) - 1;
                int board_dim = (parsed_size >= 3 && parsed_size <= 13) ? parsed_size : size;
                if (r >= 0 && r < board_dim && c >= 0 && c < board_dim) {
                    moves.push_back({r, c});
                }
            }
        } else {
            i++;
        }
    }

    if (out_first_player) *out_first_player = first_player;
    if (out_size) *out_size = parsed_size;

    return moves;
}

class AnalysisPanel {
public:
    bool is_collapsed = false;
    float panel_width = 300.0f;
    mutable std::vector<MoveTokenLayout> rendered_move_tokens;

    /**
     * Usage:
     *     AnalysisPanel panel;
     * Usage Example:
     *     panel.toggle_collapse();
     * Description:
     *     Initializes analysis panel and leaderboard display.
     */
    AnalysisPanel() = default;

    /**
     * Usage:
     *     panel.toggle_collapse();
     * Usage Example:
     *     panel.toggle_collapse();
     * Description:
     *     Toggles between expanded and collapsed pane states.
     */
    void toggle_collapse() {
        is_collapsed = !is_collapsed;
    }

    /**
     * Usage:
     *     auto cand = panel.get_hovered_candidate(mouse_x, mouse_y, win_w, top_moves);
     * Usage Example:
     *     if (auto cand = panel.get_hovered_candidate(mx, my, 1024, stats.top_moves)) { ... }
     * Description:
     *     Returns candidate move (r, c) when mouse hovers over its leaderboard row in pane.
     */
    std::optional<std::pair<int, int>> get_hovered_candidate(
        float mx, float my, float win_w, const std::vector<TopMove>& top_moves
    ) const {
        if (is_collapsed) return std::nullopt;
        float px = win_w - panel_width;
        if (mx < px || mx > win_w) return std::nullopt;

        float cur_y = 66.0f;
        for (size_t i = 0; i < top_moves.size() && i < 10; i++) {
            if (my >= cur_y && my < cur_y + 19.0f) {
                return std::make_pair(top_moves[i].r, top_moves[i].c);
            }
            cur_y += 19.0f;
        }
        return std::nullopt;
    }

    /**
     * Usage:
     *     auto token_id = panel.get_clicked_move_token(mouse_x, mouse_y);
     * Usage Example:
     *     if (auto id = panel.get_clicked_move_token(mx, my)) { tree.select_node(*id); }
     * Description:
     *     Hit-tests interactive move tokens in the PGN variation tree viewer.
     */
    std::optional<int> get_clicked_move_token(float mx, float my) const {
        if (is_collapsed) return std::nullopt;
        for (const auto& tok : rendered_move_tokens) {
            if (mx >= tok.x && mx <= tok.x + tok.w && my >= tok.y && my <= tok.y + tok.h) {
                return tok.node_id;
            }
        }
        return std::nullopt;
    }

    /**
     * Usage:
     *     panel.draw_eval_bar(ren, x, y, width, height, score);
     * Usage Example:
     *     panel.draw_eval_bar(ren, 20.0f, 70.0f, 28.0f, 480.0f, 12.5f);
     * Description:
     *     Renders vertical chess-style evaluation bar (+ for Red on bottom, - for Blue on top)
     *     with amplified non-linear scaling (reaching 2/3 bar height at +-3.0 eval) and
     *     in-bar evaluation number snapped to the winning color's edge of the division line.
     */
    void draw_eval_bar(SDL_Renderer* ren, float x, float y, float w, float h, float score) const {
        SDL_SetRenderDrawColor(ren, 50, 50, 55, 255);
        SDL_FRect bg_rect{x, y, w, h};
        SDL_RenderFillRect(ren, &bg_rect);

        // Amplified non-linear scaling: tanh curve calibrated so +-3.0 eval moves bar to 2/3rds (66.7%)
        float red_ratio = 0.5f + 0.5f * std::tanh(0.1155f * score);
        red_ratio = std::max(0.0f, std::min(1.0f, red_ratio));
        float red_h = h * red_ratio;
        float y_split = y + (h - red_h);

        // 1. Blue portion (top)
        SDL_SetRenderDrawColor(ren, 25, 118, 210, 255);
        SDL_FRect blue_rect{x, y, w, h - red_h};
        SDL_RenderFillRect(ren, &blue_rect);

        // 2. Red portion (bottom)
        SDL_SetRenderDrawColor(ren, 211, 47, 47, 255);
        SDL_FRect red_rect{x, y_split, w, red_h};
        SDL_RenderFillRect(ren, &red_rect);

        // 3. Crisp division boundary line
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 220);
        SDL_RenderLine(ren, x, y_split, x + w, y_split);

        // 4. Subtle outer border
        SDL_SetRenderDrawColor(ren, 150, 150, 160, 255);
        SDL_RenderRect(ren, &bg_rect);

        // 5. In-bar Evaluation Number snapped to the winning color's side of the division line
        std::stringstream ss;
        if (score > 90000.0f) {
            ss << "+Win";
        } else if (score < -90000.0f) {
            ss << "-Win";
        } else if (score > 0.0f) {
            ss << "+" << std::fixed << std::setprecision(1) << score;
        } else if (score < 0.0f) {
            ss << std::fixed << std::setprecision(1) << score;
        } else {
            ss << "0.0";
        }

        std::string score_str = ss.str();
        float text_w = static_cast<float>(score_str.size()) * 8.0f;
        float text_h = 10.0f;
        float text_x = x + (w - text_w) / 2.0f;

        float text_y = y_split;
        if (score > 0.0f) {
            // Winning for Red: text is located inside Red, snapped to the top of Red (riding below division line)
            text_y = y_split + 4.0f;
            if (text_y + text_h > y + h - 2.0f) {
                text_y = y + h - text_h - 2.0f;
            }
        } else if (score < 0.0f) {
            // Winning for Blue: text is located inside Blue, snapped to the bottom of Blue (riding above division line)
            text_y = y_split - text_h - 4.0f;
            if (text_y < y + 2.0f) {
                text_y = y + 2.0f;
            }
        } else {
            // Even evaluation: centered vertically on division line
            text_y = y_split - text_h / 2.0f;
        }

        // Semi-transparent dark pill backdrop for maximum legibility
        SDL_SetRenderDrawColor(ren, 15, 18, 25, 210);
        SDL_FRect pill{text_x - 3.0f, text_y - 1.0f, text_w + 6.0f, text_h + 2.0f};
        SDL_RenderFillRect(ren, &pill);

        // Sharp white evaluation number
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, text_x, text_y + 1.0f, score_str.c_str());
    }

    /**
     * Usage:
     *     auto lines = AnalysisPanelRenderer::wrap_text(text, 32);
     * Usage Example:
     *     auto lines = wrap_text("Develop central 2-bridge network and expand territory", 32);
     * Description:
     *     Splits text into multiple lines on whitespace boundaries to fit within character limits.
     */
    static std::vector<std::string> wrap_text(const std::string& text, size_t max_chars) {
        std::vector<std::string> lines;
        if (text.empty()) {
            lines.push_back("Analyzing flow...");
            return lines;
        }
        std::stringstream ss(text);
        std::string word;
        std::string current_line;

        while (ss >> word) {
            if (current_line.empty()) {
                current_line = word;
            } else if (current_line.length() + 1 + word.length() <= max_chars) {
                current_line += " " + word;
            } else {
                lines.push_back(current_line);
                current_line = word;
            }
        }
        if (!current_line.empty()) {
            lines.push_back(current_line);
        }
        return lines;
    }

    /**
     * Usage:
     *     panel.draw_panel(ren, win_width, win_height, stats, move_tree, mouse_x, mouse_y);
     * Usage Example:
     *     panel.draw_panel(ren, 1024, 720, stats, tree, mx, my);
     * Description:
     *     Renders candidate move leaderboard, strategic warden plan, live PGN branching move tree, and engine statistics.
     */
    void draw_panel(
        SDL_Renderer* ren,
        float win_w,
        float win_h,
        const EngineUIStats& stats,
        const MoveTree& tree,
        float mx = -1.0f,
        float my = -1.0f
    ) const {
        rendered_move_tokens.clear();

        if (is_collapsed) {
            float btn_x = win_w - 40.0f;
            SDL_SetRenderDrawColor(ren, 55, 60, 72, 255);
            SDL_FRect bar{btn_x, 0.0f, 40.0f, win_h};
            SDL_RenderFillRect(ren, &bar);
            SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
            SDL_RenderDebugText(ren, btn_x + 12.0f, 20.0f, "<");
            return;
        }

        float px = win_w - panel_width;
        SDL_SetRenderDrawColor(ren, 33, 37, 44, 255);
        SDL_FRect panel_rect{px, 0.0f, panel_width, win_h};
        SDL_RenderFillRect(ren, &panel_rect);

        // 1. Top Section: Top Engine Moves Leaderboard (10 moves)
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, px + 16.0f, 16.0f, "TOP ENGINE MOVES");

        SDL_SetRenderDrawColor(ren, 160, 165, 175, 255);
        SDL_RenderDebugText(ren, px + 16.0f, 42.0f, "#   Move    Eval    Depth");
        SDL_RenderLine(ren, px + 16.0f, 58.0f, px + panel_width - 16.0f, 58.0f);

        float cur_y = 66.0f;
        int rank = 1;
        for (const auto& tm : stats.top_moves) {
            char col_char = static_cast<char>('A' + tm.c);
            std::string move_str = std::string(1, col_char) + std::to_string(tm.r + 1);

            std::stringstream row_ss;
            row_ss << std::left << std::setw(3) << rank++
                   << std::setw(8) << move_str
                   << std::showpos << std::fixed << std::setprecision(1) << std::setw(8) << tm.score
                   << std::noshowpos << "d" << tm.depth;

            SDL_SetRenderDrawColor(ren, rank == 2 ? 255 : 225, rank == 2 ? 215 : 230, rank == 2 ? 64 : 240, 255);
            SDL_RenderDebugText(ren, px + 16.0f, cur_y, row_ss.str().c_str());
            cur_y += 19.0f;
            if (rank > 10) break;
        }

        // 2. Middle Section: Strategic Warden Plan & Threat Radar
        float warden_y = std::max(win_h * 0.38f, 266.0f);
        SDL_SetRenderDrawColor(ren, 160, 165, 175, 255);
        SDL_RenderLine(ren, px + 16.0f, warden_y - 6.0f, px + panel_width - 16.0f, warden_y - 6.0f);

        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, px + 16.0f, warden_y, "STRATEGIC PLAN");

        std::string threat_str = stats.threat_level >= 3 ? "[THREAT: HIGH]" : (stats.threat_level == 2 ? "[THREAT: MED]" : "[THREAT: LOW]");
        SDL_SetRenderDrawColor(ren, stats.threat_level >= 3 ? 244 : 100, stats.threat_level >= 3 ? 67 : 200, stats.threat_level >= 3 ? 54 : 255, 255);
        SDL_RenderDebugText(ren, px + 160.0f, warden_y, threat_str.c_str());

        // Dynamic multi-line text wrapping for strategic plan
        size_t max_chars = static_cast<size_t>(std::max(10.0f, (panel_width - 36.0f) / 8.0f));
        std::string raw_plan = stats.strategic_plan.empty() ? "Analyzing flow..." : stats.strategic_plan;
        std::vector<std::string> plan_lines = wrap_text(raw_plan, max_chars);

        SDL_SetRenderDrawColor(ren, 220, 225, 235, 255);
        float line_y = warden_y + 18.0f;
        for (const auto& pline : plan_lines) {
            SDL_RenderDebugText(ren, px + 16.0f, line_y, pline.c_str());
            line_y += 16.0f;
        }

        // 3. PGN Move Tree & Branching History Display (Interactive Chess Viewer Style)
        float hist_y = std::max(line_y + 8.0f, warden_y + 48.0f);
        SDL_SetRenderDrawColor(ren, 160, 165, 175, 255);
        SDL_RenderLine(ren, px + 16.0f, hist_y - 6.0f, px + panel_width - 16.0f, hist_y - 6.0f);

        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, px + 16.0f, hist_y, "GAME TREE (PGN)");

        float pgn_y = hist_y + 22.0f;
        float stat_y = win_h - 105.0f;
        float max_x = px + panel_width - 16.0f;

        if (tree.nodes.size() <= 1) {
            SDL_SetRenderDrawColor(ren, 140, 145, 155, 255);
            SDL_RenderDebugText(ren, px + 16.0f, pgn_y, "(No moves played yet)");
            SDL_SetRenderDrawColor(ren, 100, 110, 125, 255);
            SDL_RenderDebugText(ren, px + 16.0f, pgn_y + 18.0f, "[Play or use Left/Right arrows]");
        } else {
            render_tree_tokens(ren, tree, px + 16.0f, pgn_y, max_x, stat_y - 12.0f, mx, my);
        }

        // 4. Bottom Section: Engine Live Statistics
        SDL_SetRenderDrawColor(ren, 160, 165, 175, 255);
        SDL_RenderLine(ren, px + 16.0f, stat_y - 6.0f, px + panel_width - 16.0f, stat_y - 6.0f);

        std::string search_str = stats.is_searching ? "Searching..." : "Engine Idle";
        SDL_SetRenderDrawColor(ren, stats.is_searching ? 100 : 180, stats.is_searching ? 220 : 180, 100, 255);
        SDL_RenderDebugText(ren, px + 16.0f, stat_y, search_str.c_str());

        std::stringstream nodes_ss;
        nodes_ss << "Nodes: " << stats.nodes << " (" << stats.nps << " n/s)";
        SDL_SetRenderDrawColor(ren, 200, 205, 215, 255);
        SDL_RenderDebugText(ren, px + 16.0f, stat_y + 18.0f, nodes_ss.str().c_str());

        std::stringstream time_ss;
        time_ss << "Time: " << std::fixed << std::setprecision(3) << stats.elapsed_sec << "s  | Depth: " << stats.depth;
        SDL_RenderDebugText(ren, px + 16.0f, stat_y + 36.0f, time_ss.str().c_str());

        std::string best_str = "Best Move: " + stats.best_move_str;
        SDL_SetRenderDrawColor(ren, 255, 215, 64, 255);
        SDL_RenderDebugText(ren, px + 16.0f, stat_y + 54.0f, best_str.c_str());
    }

private:
    void render_tree_tokens(
        SDL_Renderer* ren,
        const MoveTree& tree,
        float start_x,
        float start_y,
        float max_x,
        float max_y,
        float mx,
        float my
    ) const {
        float cur_x = start_x;
        float cur_y = start_y;

        // Traverse tree starting from root (node 0)
        auto active_path = tree.get_path_to_current();

        std::vector<MoveTokenLayout> tokens;
        build_tokens_recursive(tree, 0, tokens);

        for (auto& tok : tokens) {
            float text_len = static_cast<float>(tok.text.size());
            float tok_w = text_len * 8.0f + 8.0f;
            float tok_h = 18.0f;

            if (cur_x + tok_w > max_x) {
                cur_x = start_x;
                cur_y += 20.0f;
            }

            if (cur_y + tok_h > max_y) {
                break; // limit to available space
            }

            tok.x = cur_x;
            tok.y = cur_y;
            tok.w = tok_w;
            tok.h = tok_h;

            bool is_hovered = (mx >= tok.x && mx <= tok.x + tok.w && my >= tok.y && my <= tok.y + tok.h);

            // Draw background pill for active or hovered token
            if (tok.is_current) {
                SDL_SetRenderDrawColor(ren, 45, 75, 120, 255);
                SDL_FRect pill{tok.x - 2.0f, tok.y - 1.0f, tok.w + 4.0f, tok.h + 2.0f};
                SDL_RenderFillRect(ren, &pill);
                SDL_SetRenderDrawColor(ren, 255, 215, 64, 255); // Gold border for current move
                SDL_RenderRect(ren, &pill);
            } else if (is_hovered) {
                SDL_SetRenderDrawColor(ren, 55, 65, 85, 200);
                SDL_FRect pill{tok.x - 2.0f, tok.y - 1.0f, tok.w + 4.0f, tok.h + 2.0f};
                SDL_RenderFillRect(ren, &pill);
                SDL_SetRenderDrawColor(ren, 120, 140, 175, 255);
                SDL_RenderRect(ren, &pill);
            }

            // Text color: Mainline moves are bright white/cream, branching variations are DARKER text (as requested)
            if (tok.is_current) {
                SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
            } else if (tok.is_branch) {
                // Darker, muted slate blue text for variations / branching paths
                SDL_SetRenderDrawColor(ren, 125, 140, 165, 255);
            } else {
                // Bright cream for mainline moves
                SDL_SetRenderDrawColor(ren, 230, 235, 245, 255);
            }

            SDL_RenderDebugText(ren, tok.x + 3.0f, tok.y + 3.0f, tok.text.c_str());
            rendered_move_tokens.push_back(tok);

            cur_x += tok_w + 4.0f;
        }
    }

    void build_tokens_recursive(
        const MoveTree& tree,
        int node_id,
        std::vector<MoveTokenLayout>& tokens,
        int depth = 0
    ) const {
        if (node_id < 0 || node_id >= static_cast<int>(tree.nodes.size())) return;
        const auto& node = tree.nodes[node_id];
        if (tokens.size() > 60) return; // Prevent excessive token count

        // 1. Process primary mainline child first
        int primary_child = -1;
        for (int ch_id : node.children) {
            if (ch_id >= 0 && ch_id < static_cast<int>(tree.nodes.size()) && !tree.nodes[ch_id].is_deleted) {
                primary_child = ch_id;
                break;
            }
        }

        if (primary_child != -1) {
            const auto& ch_node = tree.nodes[primary_child];
            MoveTokenLayout tok;
            tok.node_id = primary_child;
            tok.is_current = (primary_child == tree.current_node_id);
            tok.is_branch = (depth > 0);
            tok.is_mainline = (depth == 0);

            std::stringstream ss;
            int move_num = (ch_node.ply + 1) / 2;
            if (ch_node.ply % 2 != 0) {
                ss << move_num << "." << ch_node.to_algebraic();
            } else {
                ss << ch_node.to_algebraic();
            }
            tok.text = ss.str();
            tokens.push_back(tok);

            // 2. Process alternative branching variations for this node
            for (int alt_id : node.children) {
                if (alt_id != primary_child && alt_id >= 0 && alt_id < static_cast<int>(tree.nodes.size()) && !tree.nodes[alt_id].is_deleted) {
                    const auto& alt_node = tree.nodes[alt_id];
                    MoveTokenLayout var_tok;
                    var_tok.node_id = alt_id;
                    var_tok.is_current = (alt_id == tree.current_node_id);
                    var_tok.is_branch = true;
                    var_tok.is_mainline = false;

                    std::stringstream var_ss;
                    int var_move_num = (alt_node.ply + 1) / 2;
                    if (alt_node.ply % 2 != 0) {
                        var_ss << "(" << var_move_num << "." << alt_node.to_algebraic() << ")";
                    } else {
                        var_ss << "(" << var_move_num << ".." << alt_node.to_algebraic() << ")";
                    }
                    var_tok.text = var_ss.str();
                    tokens.push_back(var_tok);

                    // Recurse a little into variation
                    build_tokens_recursive(tree, alt_id, tokens, depth + 1);
                }
            }

            // Continue down primary mainline
            build_tokens_recursive(tree, primary_child, tokens, depth);
        }
    }
};

} // namespace hex::gui

