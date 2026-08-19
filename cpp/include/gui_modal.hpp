#pragma once

/**
 * Hex GUI Confirmation Modal Dialog Module.
 *
 * OOP Description:
 * The `hex::gui::ConfirmationModal` class handles modal dialog rendering,
 * semi-transparent overlay blending, message formatting, and button hit-testing
 * for critical user actions such as resetting the active game upon board resizing.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include <SDL3/SDL.h>
#include <string>
#include <functional>

namespace hex::gui {

class ConfirmationModal {
public:
    enum class ModalMode {
        BOARD_SIZE,
        SWAP_TURN
    };

    bool is_visible = false;
    int pending_size = 11;
    ModalMode mode = ModalMode::BOARD_SIZE;
    std::string title = "Change Board Size?";
    std::string message = "Changing board size will reset the current game. Are you sure?";

    /**
     * Usage:
     *     ConfirmationModal modal;
     * Usage Example:
     *     modal.open(9);
     * Description:
     *     Initializes confirmation modal dialog in hidden state.
     */
    ConfirmationModal() = default;

    /**
     * Usage:
     *     modal.open(new_size);
     * Usage Example:
     *     modal.open(7);
     * Description:
     *     Opens modal dialog setting pending board dimension and message.
     */
    void open(int new_size) {
        mode = ModalMode::BOARD_SIZE;
        pending_size = new_size;
        title = "Change Board Size?";
        message = "Changing board size to " + std::to_string(new_size) + "x" + std::to_string(new_size) + " will reset the game.";
        is_visible = true;
    }

    /**
     * Usage:
     *     modal.open_swap();
     * Usage Example:
     *     modal.open_swap();
     * Description:
     *     Opens modal dialog for swapping player turn order and resetting board.
     */
    void open_swap() {
        mode = ModalMode::SWAP_TURN;
        title = "Swap Turn Order?";
        message = "Swapping turn order will reset the board and start fresh.";
        is_visible = true;
    }

    /**
     * Usage:
     *     modal.close();
     * Usage Example:
     *     modal.close();
     * Description:
     *     Closes modal dialog without applying changes.
     */
    void close() {
        is_visible = false;
    }

    /**
     * Usage:
     *     int clicked = modal.handle_click(mouse_x, mouse_y, win_width, win_height);
     * Usage Example:
     *     if (modal.handle_click(mx, my, 1024, 720) == 1) { apply_resize(); }
     * Description:
     *     Checks if mouse click hit Confirm (returns 1) or Cancel (returns 2).
     */
    int handle_click(float mx, float my, float win_w, float win_h) {
        if (!is_visible) return 0;

        float box_w = 420.0f;
        float box_h = 200.0f;
        float bx = (win_w - box_w) / 2.0f;
        float by = (win_h - box_h) / 2.0f;

        float btn_y = by + 140.0f;
        float btn_w = 160.0f;
        float btn_h = 36.0f;

        float confirm_x = bx + 35.0f;
        float cancel_x = bx + 225.0f;

        // Confirm Button
        if (mx >= confirm_x && mx <= confirm_x + btn_w && my >= btn_y && my <= btn_y + btn_h) {
            is_visible = false;
            return 1;
        }

        // Cancel Button
        if (mx >= cancel_x && mx <= cancel_x + btn_w && my >= btn_y && my <= btn_y + btn_h) {
            is_visible = false;
            return 2;
        }

        return 0;
    }

    /**
     * Usage:
     *     modal.render(ren, win_width, win_height);
     * Usage Example:
     *     modal.render(ren, 1080.0f, 740.0f);
     * Description:
     *     Renders darkened overlay, dialog card, warning text, and action buttons.
     */
    void render(SDL_Renderer* ren, float win_w, float win_h) const {
        if (!is_visible) return;

        // 1. Semi-transparent backdrop overlay
        SDL_SetRenderDrawColor(ren, 10, 15, 25, 200);
        SDL_FRect backdrop{0.0f, 0.0f, win_w, win_h};
        SDL_RenderFillRect(ren, &backdrop);

        // 2. Dialog box card
        float box_w = 420.0f;
        float box_h = 200.0f;
        float bx = (win_w - box_w) / 2.0f;
        float by = (win_h - box_h) / 2.0f;

        SDL_SetRenderDrawColor(ren, 33, 37, 48, 255);
        SDL_FRect dialog_box{bx, by, box_w, box_h};
        SDL_RenderFillRect(ren, &dialog_box);

        SDL_SetRenderDrawColor(ren, 85, 95, 115, 255);
        SDL_RenderRect(ren, &dialog_box);

        // 3. Dialog Title
        SDL_SetRenderDrawColor(ren, 255, 213, 79, 255);
        SDL_RenderDebugText(ren, bx + 24.0f, by + 24.0f, title.c_str());

        // 4. Dialog Body Message
        SDL_SetRenderDrawColor(ren, 230, 235, 245, 255);
        SDL_RenderDebugText(ren, bx + 24.0f, by + 60.0f, message.c_str());
        SDL_RenderDebugText(ren, bx + 24.0f, by + 84.0f, "Are you sure you want to proceed?");

        // 5. Action Buttons
        float btn_y = by + 140.0f;
        float btn_w = 160.0f;
        float btn_h = 36.0f;
        float confirm_x = bx + 35.0f;
        float cancel_x = bx + 225.0f;

        // Confirm Button (Red Accent)
        SDL_SetRenderDrawColor(ren, 211, 47, 47, 255);
        SDL_FRect conf_rect{confirm_x, btn_y, btn_w, btn_h};
        SDL_RenderFillRect(ren, &conf_rect);
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, confirm_x + 18.0f, btn_y + 10.0f, "Confirm & Reset");

        // Cancel Button (Muted Gray)
        SDL_SetRenderDrawColor(ren, 69, 90, 100, 255);
        SDL_FRect canc_rect{cancel_x, btn_y, btn_w, btn_h};
        SDL_RenderFillRect(ren, &canc_rect);
        SDL_SetRenderDrawColor(ren, 255, 255, 255, 255);
        SDL_RenderDebugText(ren, cancel_x + 55.0f, btn_y + 10.0f, "Cancel");
    }
};

} // namespace hex::gui
