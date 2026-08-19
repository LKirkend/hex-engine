#pragma once

/**
 * Hex GUI Interactive Context Menu Dropdown Module.
 *
 * OOP Description:
 * The `hex::gui::ContextMenu` class renders a floating context menu dropdown at the
 * mouse position for move tokens in the PGN variation tree, handling option selection
 * ("Delete" / "Delete Branch" and "Make Primary Branch"), hit-testing, and visibility lifecycle.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include <SDL3/SDL.h>
#include <string>
#include <vector>
#include <algorithm>

namespace hex::gui {

struct ContextMenuItem {
    int id; // 1 = Delete / Delete Branch, 2 = Make Primary Branch
    std::string label;
};

class ContextMenu {
public:
    bool is_visible = false;
    float x = 0.0f;
    float y = 0.0f;
    float width = 165.0f;
    float item_height = 28.0f;
    int target_node_id = -1;
    std::vector<ContextMenuItem> items;

    /**
     * Usage:
     *     ContextMenu menu;
     * Usage Example:
     *     ContextMenu menu;
     * Description:
     *     Initializes context menu dropdown manager.
     */
    ContextMenu() = default;

    /**
     * Usage:
     *     context_menu.open(mx, my, node_id, has_children, win_w, win_h);
     * Usage Example:
     *     context_menu.open(200.0f, 300.0f, 4, true, 1120.0f, 750.0f);
     * Description:
     *     Opens context menu popup at (mx, my) for specified node ID.
     *     Sets option label to "Delete Branch" if target node has child moves, else "Delete".
     */
    void open(float mouse_x, float mouse_y, int node_id, bool has_children, float win_w = 1120.0f, float win_h = 750.0f) {
        target_node_id = node_id;
        is_visible = true;

        items.clear();
        std::string del_label = has_children ? "Delete Branch" : "Delete";
        items.push_back({1, del_label});
        items.push_back({2, "Make Primary Branch"});

        // Clamp inside window boundaries
        float menu_h = static_cast<float>(items.size()) * item_height + 8.0f;
        x = std::min(mouse_x, win_w - width - 10.0f);
        y = std::min(mouse_y, win_h - menu_h - 10.0f);
        x = std::max(10.0f, x);
        y = std::max(10.0f, y);
    }

    /**
     * Usage:
     *     context_menu.close();
     * Usage Example:
     *     context_menu.close();
     * Description:
     *     Closes context menu popup.
     */
    void close() {
        is_visible = false;
        target_node_id = -1;
    }

    /**
     * Usage:
     *     int action_id = context_menu.handle_click(mx, my);
     * Usage Example:
     *     int act = context_menu.handle_click(mx, my); // 1 = Delete, 2 = Make Primary, 0 = Miss/Close
     * Description:
     *     Processes mouse click on context menu items. Returns action ID if clicked, 0 otherwise.
     */
    int handle_click(float mx, float my) {
        if (!is_visible) return 0;

        float menu_h = static_cast<float>(items.size()) * item_height + 8.0f;
        if (mx < x || mx > x + width || my < y || my > y + menu_h) {
            close();
            return 0; // Clicked outside menu
        }

        float cur_y = y + 4.0f;
        for (const auto& item : items) {
            if (my >= cur_y && my < cur_y + item_height) {
                int selected_id = item.id;
                close();
                return selected_id;
            }
            cur_y += item_height;
        }

        close();
        return 0;
    }

    /**
     * Usage:
     *     context_menu.render(renderer, mouse_x, mouse_y);
     * Usage Example:
     *     context_menu.render(ren, mx, my);
     * Description:
     *     Renders dark styled context menu dropdown with hover highlights.
     */
    void render(SDL_Renderer* ren, float mx, float my) const {
        if (!is_visible) return;

        float menu_h = static_cast<float>(items.size()) * item_height + 8.0f;

        // Outer Dark Panel Background
        SDL_SetRenderDrawColor(ren, 20, 24, 33, 245);
        SDL_FRect bg_rect{x, y, width, menu_h};
        SDL_RenderFillRect(ren, &bg_rect);

        // Border Outline
        SDL_SetRenderDrawColor(ren, 80, 95, 120, 255);
        SDL_RenderRect(ren, &bg_rect);

        float cur_y = y + 4.0f;
        for (const auto& item : items) {
            bool is_hover = (mx >= x && mx <= x + width && my >= cur_y && my < cur_y + item_height);

            if (is_hover) {
                SDL_SetRenderDrawColor(ren, 48, 68, 98, 255);
                SDL_FRect item_rect{x + 2.0f, cur_y, width - 4.0f, item_height};
                SDL_RenderFillRect(ren, &item_rect);
            }

            // Colors: Red for Delete, Gold for Make Primary Branch
            if (item.id == 1) {
                SDL_SetRenderDrawColor(ren, 235, 90, 90, 255);
            } else {
                SDL_SetRenderDrawColor(ren, 255, 220, 100, 255);
            }

            SDL_RenderDebugText(ren, x + 10.0f, cur_y + 8.0f, item.label.c_str());
            cur_y += item_height;
        }
    }
};

} // namespace hex::gui
