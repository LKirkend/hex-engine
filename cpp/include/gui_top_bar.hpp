#pragma once

/**
 * Hex GUI Top Header Bar and Action Toolbar Module.
 *
 * OOP Description:
 * The `hex::gui::TopBarRenderer` class handles layout, rendering, and hit-testing
 * for the interactive top navigation bar, action buttons (Undo, Redo, Reset, Clear TT, Copy/Import PGN),
 * and incrementing number field spinners (Board Size and arbitrary Search Depth).
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include "hex_engine.hpp"
#include <SDL3/SDL.h>
#include <string>
#include <vector>

namespace hex::gui {

struct TopButton {
    std::string label;
    float x;
    float y;
    float w;
    float h;
    int id;
};

class TopBarRenderer {
public:
    /**
     * Usage:
     *     TopBarRenderer top_bar;
     * Usage Example:
     *     auto buttons = top_bar.get_buttons();
     * Description:
     *     Initializes top bar renderer and geometry layout manager.
     */
    TopBarRenderer() = default;

    /**
     * Usage:
     *     auto buttons = top_bar.get_buttons();
     * Usage Example:
     *     for (const auto& btn : top_bar.get_buttons()) { ... }
     * Description:
     *     Returns bounding geometry rectangles and action IDs for top bar buttons.
     */
    std::vector<TopButton> get_buttons() const {
        return {
            {"Undo", 10.0f, 6.0f, 54.0f, 32.0f, 1},
            {"Redo", 68.0f, 6.0f, 54.0f, 32.0f, 10},
            {"Reset", 126.0f, 6.0f, 58.0f, 32.0f, 2},
            {"Clear TT", 188.0f, 6.0f, 82.0f, 32.0f, 3},
            {"Copy PGN", 274.0f, 6.0f, 82.0f, 32.0f, 4},
            {"Import PGN", 360.0f, 6.0f, 98.0f, 32.0f, 5},
            // Board Size Spinner [-] N [+]
            {"-", 518.0f, 6.0f, 26.0f, 32.0f, 6},
            {"+", 576.0f, 6.0f, 26.0f, 32.0f, 7},
            // Search Depth Spinner [-] D [+] (arbitrarily high)
            {"-", 668.0f, 6.0f, 26.0f, 32.0f, 8},
            {"+", 726.0f, 6.0f, 26.0f, 32.0f, 9},
            // Swap Turn Button
            {"Swap Turn", 764.0f, 6.0f, 92.0f, 32.0f, 11},
        };
    }

    /**
     * Usage:
     *     top_bar.render(ren, win_w, board_size, depth, current_player, winner);
     * Usage Example:
     *     top_bar.render(ren, 1120.0f, 11, 20, hex::BLUE, hex::EMPTY);
     * Description:
     *     Renders dark header bar, action buttons, size/depth spinners, and player turn badge.
     */
    void render(
        SDL_Renderer* ren,
        float win_w,
        int board_size,
        int depth,
        uint8_t current_player,
        uint8_t winner
    ) const {
        // 1. Top Bar Background
        SDL_SetRenderDrawColor(ren, 25, 30, 40, 255);
        SDL_FRect bar_rect{0.0f, 0.0f, win_w, 44.0f};
        SDL_RenderFillRect(ren, &bar_rect);

        // 2. Render Action Buttons with Perfect Centering
        for (const auto& btn : get_buttons()) {
            SDL_SetRenderDrawColor(ren, 50, 58, 75, 255);
            SDL_FRect b_rect{btn.x, btn.y, btn.w, btn.h};
            SDL_RenderFillRect(ren, &b_rect);

            SDL_SetRenderDrawColor(ren, 80, 92, 118, 255);
            SDL_RenderRect(ren, &b_rect);

            // Centered text inside button geometry
            float text_w = static_cast<float>(btn.label.length() * 8);
            float text_x = btn.x + (btn.w - text_w) * 0.5f;
            float text_y = btn.y + (btn.h - 8.0f) * 0.5f;

            SDL_SetRenderDrawColor(ren, 240, 245, 255, 255);
            SDL_RenderDebugText(ren, text_x, text_y, btn.label.c_str());
        }

        // 3. Render Size and Depth Spinner Labels & Badges
        SDL_SetRenderDrawColor(ren, 180, 190, 205, 255);
        SDL_RenderDebugText(ren, 470.0f, 18.0f, "Size:");
        std::string size_val = std::to_string(board_size);
        float size_val_w = static_cast<float>(size_val.length() * 8);
        float size_val_x = 518.0f + 26.0f + (576.0f - (518.0f + 26.0f) - size_val_w) * 0.5f;
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, size_val_x, 18.0f, size_val.c_str());

        SDL_SetRenderDrawColor(ren, 180, 190, 205, 255);
        SDL_RenderDebugText(ren, 614.0f, 18.0f, "Depth:");
        std::string depth_val = std::to_string(depth);
        float depth_val_w = static_cast<float>(depth_val.length() * 8);
        float depth_val_x = 668.0f + 26.0f + (726.0f - (668.0f + 26.0f) - depth_val_w) * 0.5f;
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, depth_val_x, 18.0f, depth_val.c_str());

        // 4. Status Badge on Right Side
        std::string status_str;
        if (winner == RED) {
            status_str = "RED WON!";
            SDL_SetRenderDrawColor(ren, 211, 47, 47, 255);
        } else if (winner == BLUE) {
            status_str = "BLUE WON!";
            SDL_SetRenderDrawColor(ren, 25, 118, 210, 255);
        } else {
            status_str = (current_player == RED ? "Turn: RED" : "Turn: BLUE");
            SDL_SetRenderDrawColor(ren, current_player == RED ? 211 : 25, current_player == RED ? 47 : 118, current_player == RED ? 47 : 210, 255);
        }
        float badge_w = 95.0f;
        float badge_h = 28.0f;
        float badge_x = win_w - 180.0f;
        float badge_y = 8.0f;
        SDL_FRect turn_badge{badge_x, badge_y, badge_w, badge_h};
        SDL_RenderFillRect(ren, &turn_badge);
        float badge_text_w = static_cast<float>(status_str.length() * 8);
        float badge_text_x = badge_x + (badge_w - badge_text_w) * 0.5f;
        float badge_text_y = badge_y + (badge_h - 8.0f) * 0.5f;
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, badge_text_x, badge_text_y, status_str.c_str());
    }
};

} // namespace hex::gui
