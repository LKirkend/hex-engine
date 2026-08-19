#pragma once

/**
 * Hex Diamond Board Graphical Renderer Module.
 *
 * OOP Description:
 * The `hex::gui::BoardRenderer` class handles 2D geometric diamond projection,
 * flat-topped hexagon rasterization, thick geometry-hugging borders, 4-sided edge
 * coordinate labeling, stone rendering, and candidate move ghost previews on SDL3.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include "hex_engine.hpp"
#include <SDL3/SDL.h>
#include <cmath>
#include <vector>
#include <string>
#include <optional>
#include <algorithm>

namespace hex::gui {

struct Point2D {
    float x;
    float y;
};

class BoardRenderer {
public:
    float offset_x = 180.0f;
    float offset_y = 380.0f;
    float hex_radius = 20.0f;

    /**
     * Usage:
     *     BoardRenderer renderer;
     * Usage Example:
     *     BoardRenderer renderer;
     * Description:
     *     Initializes diamond board coordinate renderer with default geometry.
     */
    BoardRenderer() = default;

    /**
     * Usage:
     *     renderer.update_layout(win_width, win_height, board_size, is_collapsed);
     * Usage Example:
     *     renderer.update_layout(1024.0f, 720.0f, 11, false);
     * Description:
     *     Calculates optimal radius and centering offsets so diamond board fills canvas.
     */
    void update_layout(float win_w, float win_h, int size, bool is_collapsed) {
        float right_pane_w = is_collapsed ? 40.0f : 300.0f;
        float left_margin = 80.0f;
        float avail_w = win_w - right_pane_w - left_margin - 30.0f;
        float avail_h = win_h - 75.0f;

        float sqrt3 = std::sqrt(3.0f);
        float r_w = avail_w / std::max(1.0f, (3.0f * size - 1.0f));
        float r_h = avail_h / std::max(1.0f, ((size + 0.8f) * sqrt3));

        hex_radius = std::min(r_w, r_h);
        hex_radius = std::max(12.0f, hex_radius);

        float grid_w = (2 * size - 2) * 1.5f * hex_radius;
        offset_x = left_margin + (avail_w - grid_w) / 2.0f;
        offset_y = 48.0f + avail_h / 2.0f;
    }

    /**
     * Usage:
     *     Point2D pt = renderer.get_hex_center(r, c);
     * Usage Example:
     *     auto [x, y] = renderer.get_hex_center(5, 5);
     * Description:
     *     Calculates screen pixel center (x, y) for diamond hex grid coordinates (r, c).
     *     A1 (0, 0) is at Far Left, K1 (0, N-1) at Top, A11 (N-1, 0) at Bottom, K11 (N-1, N-1) at Right.
     */
    Point2D get_hex_center(int r, int c) const {
        float sqrt3 = std::sqrt(3.0f);
        float x = offset_x + (r + c) * (1.5f * hex_radius);
        float y = offset_y + (r - c) * (sqrt3 / 2.0f * hex_radius);
        return Point2D{x, y};
    }

    /**
     * Usage:
     *     auto coord = renderer.pixel_to_hex(px, py, board_size);
     * Usage Example:
     *     if (auto cell = renderer.pixel_to_hex(mouse_x, mouse_y, 11)) { ... }
     * Description:
     *     Converts screen pixel coordinates back into (row, col) diamond grid cell.
     */
    std::optional<std::pair<int, int>> pixel_to_hex(float px, float py, int size) const {
        float min_dist_sq = (hex_radius * 1.0f) * (hex_radius * 1.0f);
        std::optional<std::pair<int, int>> best_cell = std::nullopt;

        for (int r = 0; r < size; r++) {
            for (int c = 0; c < size; c++) {
                Point2D center = get_hex_center(r, c);
                float dx = px - center.x;
                float dy = py - center.y;
                float d_sq = dx * dx + dy * dy;
                if (d_sq < min_dist_sq) {
                    min_dist_sq = d_sq;
                    best_cell = std::make_pair(r, c);
                }
            }
        }
        return best_cell;
    }

    /**
     * Usage:
     *     renderer.draw_board(sdl_renderer, board, hover_cell, top_moves, selected_candidate);
     * Usage Example:
     *     renderer.draw_board(ren, board, hover_cell, top_moves, std::nullopt);
     * Description:
     *     Renders diamond hexagon cells, hugging thick borders, placed stones, and ghost previews.
     */
    void draw_board(
        SDL_Renderer* ren,
        const HexBoard& board,
        std::optional<std::pair<int, int>> hover_cell,
        const std::vector<TopMove>& top_moves,
        std::optional<std::pair<int, int>> selected_candidate,
        const std::vector<std::pair<int, int>>& book_moves = {}
    ) const {
        int s = board.size;

        // 1. Draw hexagonal cells and placed stones
        for (int r = 0; r < s; r++) {
            for (int c = 0; c < s; c++) {
                Point2D center = get_hex_center(r, c);
                uint8_t cell = board.grid[r * s + c];

                // Draw hexagon outline
                draw_hexagon(ren, center.x, center.y, hex_radius, 200, 205, 215);

                if (cell == RED) {
                    draw_filled_circle(ren, center.x, center.y, hex_radius * 0.72f, 211, 47, 47); // #D32F2F
                } else if (cell == BLUE) {
                    draw_filled_circle(ren, center.x, center.y, hex_radius * 0.72f, 25, 118, 210); // #1976D2
                }
            }
        }

        // 2. Draw Opening Book Ghost Stones with Negative Cutout Book Icons
        for (const auto& [br, bc] : book_moves) {
            if (br >= 0 && br < s && bc >= 0 && bc < s && board.grid[br * s + bc] == EMPTY) {
                Point2D center = get_hex_center(br, bc);
                if (board.current_player == RED) {
                    draw_filled_circle(ren, center.x, center.y, hex_radius * 0.65f, 229, 115, 115);
                } else {
                    draw_filled_circle(ren, center.x, center.y, hex_radius * 0.65f, 100, 181, 246);
                }
                draw_book_icon(ren, center.x, center.y, hex_radius * 0.38f);
            }
        }

        // 3. Draw Candidate Moves Ghost Previews
        for (size_t i = 0; i < top_moves.size() && i < 3; i++) {
            const auto& tm = top_moves[i];
            if (tm.r >= 0 && tm.r < s && tm.c >= 0 && tm.c < s) {
                if (board.grid[tm.r * s + tm.c] == EMPTY) {
                    Point2D center = get_hex_center(tm.r, tm.c);
                    if (i == 0) {
                        // Rank 1 Best Move Highlight
                        draw_dashed_ring(ren, center.x, center.y, hex_radius * 0.75f, 255, 213, 79);
                        SDL_SetRenderDrawColor(ren, 180, 140, 20, 255);
                        SDL_RenderDebugText(ren, center.x - 4.0f, center.y - 4.0f, "#1");
                    } else {
                        // Rank 2 & 3 Moves
                        draw_dashed_ring(ren, center.x, center.y, hex_radius * 0.60f, 180, 190, 205);
                    }
                }
            }
        }

        // 4. Draw Selected / Hovered Candidate from pane
        if (selected_candidate.has_value()) {
            auto [cr, cc] = selected_candidate.value();
            if (cr >= 0 && cr < s && cc >= 0 && cc < s && board.grid[cr * s + cc] == EMPTY) {
                Point2D center = get_hex_center(cr, cc);
                draw_filled_circle(ren, center.x, center.y, hex_radius * 0.65f, 255, 235, 59);
            }
        }

        // 5. Draw Mouse Hover Ghost Stone
        if (hover_cell.has_value()) {
            auto [hr, hc] = hover_cell.value();
            if (hr >= 0 && hr < s && hc >= 0 && hc < s && board.grid[hr * s + hc] == EMPTY) {
                Point2D center = get_hex_center(hr, hc);
                if (board.current_player == RED) {
                    draw_filled_circle(ren, center.x, center.y, hex_radius * 0.60f, 255, 205, 210);
                } else {
                    draw_filled_circle(ren, center.x, center.y, hex_radius * 0.60f, 187, 222, 251);
                }
            }
        }

        // 6. Draw thick outer borders hugging hexagon geometry
        draw_thick_hugging_borders(ren, s);

        // 7. Draw coordinate labels on all 4 sides
        draw_border_labels(ren, s);
    }

private:
    /**
     * Usage:
     *     draw_book_icon(ren, center.x, center.y, hex_radius * 0.40f);
     * Usage Example:
     *     draw_book_icon(ren, cx, cy, 14.0f);
     * Description:
     *     Renders a crisp negative cutout of an open book symbol (two open wings/pages with center spine).
     */
    void draw_book_icon(SDL_Renderer* ren, float cx, float cy, float size) const {
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 245);
        float w = size * 1.25f;
        float h = size * 0.95f;

        // Center spine
        SDL_RenderLine(ren, cx, cy - h * 0.5f, cx, cy + h * 0.5f);

        // Left Page
        SDL_RenderLine(ren, cx, cy - h * 0.5f, cx - w * 0.5f, cy - h * 0.35f);
        SDL_RenderLine(ren, cx - w * 0.5f, cy - h * 0.35f, cx - w * 0.5f, cy + h * 0.45f);
        SDL_RenderLine(ren, cx - w * 0.5f, cy + h * 0.45f, cx, cy + h * 0.5f);

        // Right Page
        SDL_RenderLine(ren, cx, cy - h * 0.5f, cx + w * 0.5f, cy - h * 0.35f);
        SDL_RenderLine(ren, cx + w * 0.5f, cy - h * 0.35f, cx + w * 0.5f, cy + h * 0.45f);
        SDL_RenderLine(ren, cx + w * 0.5f, cy + h * 0.45f, cx, cy + h * 0.5f);

        // Subtle interior line detail
        SDL_RenderLine(ren, cx - w * 0.35f, cy - h * 0.08f, cx - w * 0.12f, cy - h * 0.12f);
        SDL_RenderLine(ren, cx - w * 0.35f, cy + h * 0.14f, cx - w * 0.12f, cy + h * 0.10f);
        SDL_RenderLine(ren, cx + w * 0.12f, cy - h * 0.12f, cx + w * 0.35f, cy - h * 0.08f);
        SDL_RenderLine(ren, cx + w * 0.12f, cy + h * 0.10f, cx + w * 0.35f, cy + h * 0.14f);
    }
    void draw_thick_line(SDL_Renderer* ren, float x1, float y1, float x2, float y2, int thickness, uint8_t r, uint8_t g, uint8_t b) const {
        SDL_SetRenderDrawColor(ren, r, g, b, 255);
        int half = thickness / 2;
        for (int dx = -half; dx <= half; dx++) {
            for (int dy = -half; dy <= half; dy++) {
                SDL_RenderLine(ren, x1 + dx, y1 + dy, x2 + dx, y2 + dy);
            }
        }
    }

    void draw_thick_hugging_borders(SDL_Renderer* ren, int s) const {
        float sqrt3 = std::sqrt(3.0f);
        float h_half = sqrt3 / 2.0f * hex_radius;

        // Top-Left Red Border (along row 0, cols 0..s-1)
        for (int c = 0; c < s; c++) {
            Point2D pt = get_hex_center(0, c);
            Point2D p_left{pt.x - hex_radius, pt.y};
            Point2D p_top_left{pt.x - 0.5f * hex_radius, pt.y - h_half};
            Point2D p_top_right{pt.x + 0.5f * hex_radius, pt.y - h_half};

            draw_thick_line(ren, p_left.x, p_left.y, p_top_left.x, p_top_left.y, 4, 211, 47, 47);
            draw_thick_line(ren, p_top_left.x, p_top_left.y, p_top_right.x, p_top_right.y, 4, 211, 47, 47);
        }

        // Bottom-Right Red Border (along row s-1, cols 0..s-1)
        for (int c = 0; c < s; c++) {
            Point2D pt = get_hex_center(s - 1, c);
            Point2D p_bot_left{pt.x - 0.5f * hex_radius, pt.y + h_half};
            Point2D p_bot_right{pt.x + 0.5f * hex_radius, pt.y + h_half};
            Point2D p_right{pt.x + hex_radius, pt.y};

            draw_thick_line(ren, p_bot_left.x, p_bot_left.y, p_bot_right.x, p_bot_right.y, 4, 211, 47, 47);
            draw_thick_line(ren, p_bot_right.x, p_bot_right.y, p_right.x, p_right.y, 4, 211, 47, 47);
        }

        // Bottom-Left Blue Border (along col 0, rows 0..s-1)
        for (int r = 0; r < s; r++) {
            Point2D pt = get_hex_center(r, 0);
            Point2D p_left{pt.x - hex_radius, pt.y};
            Point2D p_bot_left{pt.x - 0.5f * hex_radius, pt.y + h_half};
            Point2D p_bot_right{pt.x + 0.5f * hex_radius, pt.y + h_half};

            draw_thick_line(ren, p_left.x, p_left.y, p_bot_left.x, p_bot_left.y, 4, 25, 118, 210);
            draw_thick_line(ren, p_bot_left.x, p_bot_left.y, p_bot_right.x, p_bot_right.y, 4, 25, 118, 210);
        }

        // Top-Right Blue Border (along col s-1, rows 0..s-1)
        for (int r = 0; r < s; r++) {
            Point2D pt = get_hex_center(r, s - 1);
            Point2D p_top_left{pt.x - 0.5f * hex_radius, pt.y - h_half};
            Point2D p_top_right{pt.x + 0.5f * hex_radius, pt.y - h_half};
            Point2D p_right{pt.x + hex_radius, pt.y};

            draw_thick_line(ren, p_top_left.x, p_top_left.y, p_top_right.x, p_top_right.y, 4, 25, 118, 210);
            draw_thick_line(ren, p_top_right.x, p_top_right.y, p_right.x, p_right.y, 4, 25, 118, 210);
        }
    }

    void draw_border_labels(SDL_Renderer* ren, int s) const {
        float sqrt3 = std::sqrt(3.0f);

        // 1. Top-Left Red Edge Labels (A-K along row 0): moved left 5px, down 5px
        SDL_SetRenderDrawColor(ren, 211, 47, 47, 255);
        for (int c = 0; c < s; c++) {
            Point2D pt = get_hex_center(0, c);
            char col_char[2] = {static_cast<char>('A' + c), '\0'};
            float lx = pt.x - 0.75f * hex_radius - 17.0f;
            float ly = pt.y - (sqrt3 / 4.0f * hex_radius) - 5.0f;
            SDL_RenderDebugText(ren, lx, ly, col_char);
        }

        // 2. Bottom-Left Blue Edge Labels (1-N along col 0): right-aligned and moved 2px left to prevent 10/11 edge clipping
        SDL_SetRenderDrawColor(ren, 25, 118, 210, 255);
        for (int r = 0; r < s; r++) {
            Point2D pt = get_hex_center(r, 0);
            std::string row_str = std::to_string(r + 1);
            float text_w = static_cast<float>(row_str.size() * 8);
            float base_right = pt.x - 0.75f * hex_radius - 13.0f; // Right edge anchor (2px left of previous)
            float lx = base_right - text_w;                      // Right-aligned
            float ly = pt.y + (sqrt3 / 4.0f * hex_radius) - 2.0f;
            SDL_RenderDebugText(ren, lx, ly, row_str.c_str());
        }

        // 3. Bottom-Right Red Edge Labels (A-K along row s-1): moved left 5px, up 10px
        SDL_SetRenderDrawColor(ren, 211, 47, 47, 255);
        for (int c = 0; c < s; c++) {
            Point2D pt = get_hex_center(s - 1, c);
            char col_char[2] = {static_cast<char>('A' + c), '\0'};
            float lx = pt.x + 0.75f * hex_radius + 7.0f;
            float ly = pt.y + (sqrt3 / 4.0f * hex_radius) - 2.0f;
            SDL_RenderDebugText(ren, lx, ly, col_char);
        }

        // 4. Top-Right Blue Edge Labels (1-N along col s-1): moved left 5px, down 5px
        SDL_SetRenderDrawColor(ren, 25, 118, 210, 255);
        for (int r = 0; r < s; r++) {
            Point2D pt = get_hex_center(r, s - 1);
            std::string row_str = std::to_string(r + 1);
            float lx = pt.x + 0.75f * hex_radius + 7.0f;
            float ly = pt.y - (sqrt3 / 4.0f * hex_radius) - 5.0f;
            SDL_RenderDebugText(ren, lx, ly, row_str.c_str());
        }
    }

    void draw_hexagon(SDL_Renderer* ren, float cx, float cy, float r, uint8_t red, uint8_t green, uint8_t blue) const {
        SDL_SetRenderDrawColor(ren, red, green, blue, 255);
        Point2D pts[6];
        for (int i = 0; i < 6; i++) {
            float angle = (i * 60.0f) * (3.14159265f / 180.0f);
            pts[i] = Point2D{cx + r * std::cos(angle), cy + r * std::sin(angle)};
        }
        for (int i = 0; i < 6; i++) {
            int j = (i + 1) % 6;
            SDL_RenderLine(ren, pts[i].x, pts[i].y, pts[j].x, pts[j].y);
        }
    }

    void draw_filled_circle(SDL_Renderer* ren, float cx, float cy, float r, uint8_t red, uint8_t green, uint8_t blue) const {
        SDL_SetRenderDrawColor(ren, red, green, blue, 255);
        int radius = static_cast<int>(r);
        for (int w = 0; w < radius * 2; w++) {
            for (int h = 0; h < radius * 2; h++) {
                int dx = radius - w;
                int dy = radius - h;
                if ((dx * dx + dy * dy) <= (radius * radius)) {
                    SDL_RenderPoint(ren, cx + dx, cy + dy);
                }
            }
        }
    }

    void draw_dashed_ring(SDL_Renderer* ren, float cx, float cy, float r, uint8_t red, uint8_t green, uint8_t blue) const {
        SDL_SetRenderDrawColor(ren, red, green, blue, 255);
        int segments = 16;
        for (int i = 0; i < segments; i += 2) {
            float a1 = (i * 360.0f / segments) * (3.14159265f / 180.0f);
            float a2 = ((i + 1) * 360.0f / segments) * (3.14159265f / 180.0f);
            SDL_RenderLine(ren, cx + r * std::cos(a1), cy + r * std::sin(a1),
                                cx + r * std::cos(a2), cy + r * std::sin(a2));
        }
    }
};

} // namespace hex::gui
