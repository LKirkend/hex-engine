#pragma once

/**
 * Hex Move Tree & Variation Management Module.
 *
 * OOP Description:
 * The `hex::gui::MoveNode` and `hex::gui::MoveTree` classes implement a game-tree
 * data structure supporting multi-branch variation paths, interactive navigation
 * (undo/redo, left/right arrow keys, direct node selection), branch deletion
 * via context interaction (right-click), and PGN serialization.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include <vector>
#include <string>
#include <sstream>
#include <iomanip>
#include <algorithm>
#include <cstdint>
#include <optional>

namespace hex::gui {

/**
 * Node within a game tree representing a single placed stone.
 */
struct MoveNode {
    int id = 0;
    int parent_id = -1;
    int r = 0;
    int c = 0;
    uint8_t player = 0;
    int ply = 0;
    int depth = 0;
    double elapsed_sec = 0.0;
    float eval_score = 0.0f;
    std::vector<int> children;
    bool is_deleted = false;

    /**
     * Usage:
     *     std::string str = node.to_algebraic();
     * Usage Example:
     *     auto str = node.to_algebraic(); // "f6"
     * Description:
     *     Converts move coordinate (r, c) into algebraic notation (e.g. "f6", "k1").
     */
    std::string to_algebraic() const {
        char col_char = static_cast<char>('a' + c);
        return std::string(1, col_char) + std::to_string(r + 1);
    }
};

/**
 * Clickable visual token representation for rendering and hit-testing in the UI.
 */
struct MoveTokenLayout {
    int node_id = 0;
    std::string text = "";
    float x = 0.0f;
    float y = 0.0f;
    float w = 0.0f;
    float h = 0.0f;
    bool is_current = false;
    bool is_branch = false;
    bool is_mainline = true;
};

/**
 * Usage:
 *     bool ok = parse_pgn_tag("[First \"Red\"]", key, val);
 * Usage Example:
 *     std::string k, v;
 *     if (parse_pgn_tag(tag_str, k, v)) { ... }
 * Description:
 *     Parses a PGN tag pair string of format `[Key "Value"]` into key and value substrings.
 */
inline bool parse_pgn_tag(const std::string& tag_str, std::string& out_key, std::string& out_val) {
    size_t start = tag_str.find('[');
    size_t end = tag_str.rfind(']');
    if (start == std::string::npos || end == std::string::npos || end <= start) return false;
    std::string inner = tag_str.substr(start + 1, end - start - 1);

    size_t q1 = inner.find('"');
    size_t q2 = inner.rfind('"');
    if (q1 == std::string::npos || q2 == std::string::npos || q1 >= q2) return false;

    out_key = inner.substr(0, q1);
    out_key.erase(0, out_key.find_first_not_of(" \t\r\n"));
    size_t last_k = out_key.find_last_not_of(" \t\r\n");
    if (last_k != std::string::npos) out_key.erase(last_k + 1);

    out_val = inner.substr(q1 + 1, q2 - q1 - 1);
    return true;
}

/**
 * Usage:
 *     process_pgn_tag(key, val, player, board_size);
 * Usage Example:
 *     process_pgn_tag("First", "Red", player, size);
 * Description:
 *     Processes extracted PGN tag key/value pairs for player starting color and board dimension.
 */
inline void process_pgn_tag(const std::string& key, const std::string& val, uint8_t& player, int& board_size) {
    std::string k_lower = key;
    std::transform(k_lower.begin(), k_lower.end(), k_lower.begin(), [](unsigned char c) { return static_cast<char>(std::tolower(c)); });

    std::string v_lower = val;
    std::transform(v_lower.begin(), v_lower.end(), v_lower.begin(), [](unsigned char c) { return static_cast<char>(std::tolower(c)); });

    if (k_lower == "first" || k_lower == "firstplayer" || k_lower == "startingplayer" ||
        k_lower == "player1" || k_lower == "turn" || k_lower == "color" || k_lower == "side") {
        if (!v_lower.empty()) {
            if (v_lower[0] == 'r' || v_lower[0] == 'w' || v_lower == "1") {
                player = 1; // RED
            } else if (v_lower[0] == 'b' || v_lower == "2") {
                player = 2; // BLUE
            }
        }
    } else if (k_lower == "size" || k_lower == "boardsize") {
        try {
            int s = std::stoi(val);
            if (s >= 3 && s <= 13) {
                board_size = s;
            }
        } catch (...) {}
    }
}

/**
 * Tree manager maintaining full game variations, branch navigation, and node life-cycles.
 */
class MoveTree {
public:
    std::vector<MoveNode> nodes;
    int current_node_id = 0;

    /**
     * Usage:
     *     MoveTree tree;
     * Usage Example:
     *     MoveTree tree;
     * Description:
     *     Initializes game move tree with a single root node (empty board).
     */
    MoveTree() {
        clear();
    }

    /**
     * Usage:
     *     tree.clear();
     * Usage Example:
     *     tree.clear();
     * Description:
     *     Resets tree to root node at ply 0.
     */
    void clear() {
        nodes.clear();
        MoveNode root;
        root.id = 0;
        root.parent_id = -1;
        root.r = -1;
        root.c = -1;
        root.player = 0;
        root.ply = 0;
        nodes.push_back(root);
        current_node_id = 0;
    }

    /**
     * Usage:
     *     int id = tree.add_or_select_move(r, c, player, depth, elapsed_sec, eval_score);
     * Usage Example:
     *     int id = tree.add_or_select_move(5, 5, RED, 14, 0.450, 1.2f);
     * Description:
     *     Adds move as a child of the current node if not already present,
     *     or navigates to the existing child, returning the new current node ID.
     *     Records calculation search depth, elapsed calculation time, and eval score.
     */
    int add_or_select_move(int r, int c, uint8_t player, int depth = 0, double elapsed_sec = 0.0, float eval_score = 0.0f) {
        if (current_node_id < 0 || current_node_id >= static_cast<int>(nodes.size())) {
            current_node_id = 0;
        }

        // Check if move already exists among active children
        for (int child_id : nodes[current_node_id].children) {
            if (child_id >= 0 && child_id < static_cast<int>(nodes.size())) {
                auto& child = nodes[child_id];
                if (!child.is_deleted && child.r == r && child.c == c) {
                    if (depth > 0) child.depth = depth;
                    if (elapsed_sec > 0.0) child.elapsed_sec = elapsed_sec;
                    if (eval_score != 0.0f) child.eval_score = eval_score;
                    current_node_id = child_id;
                    return child_id;
                }
            }
        }

        // Create new child node
        int new_id = static_cast<int>(nodes.size());
        MoveNode new_node;
        new_node.id = new_id;
        new_node.parent_id = current_node_id;
        new_node.r = r;
        new_node.c = c;
        new_node.player = player;
        new_node.ply = nodes[current_node_id].ply + 1;
        new_node.depth = depth;
        new_node.elapsed_sec = elapsed_sec;
        new_node.eval_score = eval_score;
        new_node.is_deleted = false;

        nodes.push_back(new_node);
        nodes[current_node_id].children.push_back(new_id);
        current_node_id = new_id;
        return new_id;
    }


    /**
     * Usage:
     *     bool ok = tree.step_backward();
     * Usage Example:
     *     if (tree.step_backward()) { rebuild_board(); }
     * Description:
     *     Navigates to parent node (Undo / Left Arrow). Returns true if stepped.
     */
    bool step_backward() {
        if (current_node_id > 0 && current_node_id < static_cast<int>(nodes.size())) {
            current_node_id = nodes[current_node_id].parent_id;
            return true;
        }
        return false;
    }

    /**
     * Usage:
     *     bool ok = tree.step_forward();
     * Usage Example:
     *     if (tree.step_forward()) { rebuild_board(); }
     * Description:
     *     Navigates to first active child node (Redo / Right Arrow). Returns true if stepped.
     */
    bool step_forward() {
        if (current_node_id >= 0 && current_node_id < static_cast<int>(nodes.size())) {
            for (int child_id : nodes[current_node_id].children) {
                if (child_id >= 0 && child_id < static_cast<int>(nodes.size()) && !nodes[child_id].is_deleted) {
                    current_node_id = child_id;
                    return true;
                }
            }
        }
        return false;
    }

    /**
     * Usage:
     *     bool ok = tree.select_node(target_id);
     * Usage Example:
     *     tree.select_node(3);
     * Description:
     *     Navigates directly to any non-deleted node in the tree.
     */
    bool select_node(int target_id) {
        if (target_id >= 0 && target_id < static_cast<int>(nodes.size()) && !nodes[target_id].is_deleted) {
            current_node_id = target_id;
            return true;
        }
        return false;
    }

    /**
     * Usage:
     *     bool has_children = tree.has_following_nodes(node_id);
     * Usage Example:
     *     if (tree.has_following_nodes(4)) { ... }
     * Description:
     *     Returns true if node has one or more non-deleted child nodes.
     */
    bool has_following_nodes(int node_id) const {
        if (node_id >= 0 && node_id < static_cast<int>(nodes.size())) {
            for (int ch_id : nodes[node_id].children) {
                if (ch_id >= 0 && ch_id < static_cast<int>(nodes.size()) && !nodes[ch_id].is_deleted) {
                    return true;
                }
            }
        }
        return false;
    }

    /**
     * Usage:
     *     bool ok = tree.make_primary_branch(target_id);
     * Usage Example:
     *     tree.make_primary_branch(4);
     * Description:
     *     Promotes the selected variation node (and its ancestors) to be the first child (primary branch)
     *     at each parent node up to the root.
     */
    bool make_primary_branch(int target_id) {
        if (target_id <= 0 || target_id >= static_cast<int>(nodes.size()) || nodes[target_id].is_deleted) {
            return false;
        }

        int curr = target_id;
        while (curr > 0 && curr < static_cast<int>(nodes.size())) {
            int p_id = nodes[curr].parent_id;
            if (p_id >= 0 && p_id < static_cast<int>(nodes.size())) {
                auto& ch = nodes[p_id].children;
                auto it = std::find(ch.begin(), ch.end(), curr);
                if (it != ch.end() && it != ch.begin()) {
                    ch.erase(it);
                    ch.insert(ch.begin(), curr);
                }
            }
            curr = p_id;
        }
        return true;
    }

    /**
     * Usage:
     *     bool ok = tree.delete_branch(target_id);
     * Usage Example:
     *     tree.delete_branch(4); // removes branch starting at node 4
     * Description:
     *     Deletes branch starting at target_id and all its descendants (Right-click remove).
     *     If active position was inside the deleted branch, navigates to target's parent.
     */
    bool delete_branch(int target_id) {
        if (target_id <= 0 || target_id >= static_cast<int>(nodes.size()) || nodes[target_id].is_deleted) {
            return false;
        }

        // If current node is within subtree being deleted, move to parent of deleted node
        if (is_descendant_of(current_node_id, target_id) || current_node_id == target_id) {
            current_node_id = nodes[target_id].parent_id;
        }

        // Remove target_id from parent's children list
        int parent_id = nodes[target_id].parent_id;
        if (parent_id >= 0 && parent_id < static_cast<int>(nodes.size())) {
            auto& ch = nodes[parent_id].children;
            ch.erase(std::remove(ch.begin(), ch.end(), target_id), ch.end());
        }

        // Mark target and descendants as deleted
        mark_subtree_deleted(target_id);
        return true;
    }

    /**
     * Usage:
     *     auto path = tree.get_path_to_current();
     * Usage Example:
     *     for (const auto& [r, c] : tree.get_path_to_current()) { ... }
     * Description:
     *     Returns linear list of (r, c) moves leading from root to the current active node.
     */
    std::vector<std::pair<int, int>> get_path_to_current() const {
        return get_path_to_node(current_node_id);
    }

    /**
     * Usage:
     *     auto path = tree.get_path_to_node(node_id);
     * Usage Example:
     *     auto path = tree.get_path_to_node(5);
     * Description:
     *     Returns linear list of (r, c) moves leading from root to specified node.
     */
    std::vector<std::pair<int, int>> get_path_to_node(int node_id) const {
        std::vector<std::pair<int, int>> path;
        int curr = node_id;
        while (curr > 0 && curr < static_cast<int>(nodes.size())) {
            const auto& node = nodes[curr];
            if (node.is_deleted) break;
            path.push_back({node.r, node.c});
            curr = node.parent_id;
        }
        std::reverse(path.begin(), path.end());
        return path;
    }

    /**
     * Usage:
     *     bool empty = tree.is_at_root();
     * Usage Example:
     *     if (tree.is_at_root()) { ... }
     * Description:
     *     Returns true if current active position is root (0 moves).
     */
    bool is_at_root() const {
        return current_node_id == 0;
    }

    /**
     * Usage:
     *     bool can_redo = tree.can_step_forward();
     * Usage Example:
     *     if (tree.can_step_forward()) { ... }
     * Description:
     *     Returns true if active node has child moves available to step into.
     */
    bool can_step_forward() const {
        if (current_node_id >= 0 && current_node_id < static_cast<int>(nodes.size())) {
            for (int child_id : nodes[current_node_id].children) {
                if (child_id >= 0 && child_id < static_cast<int>(nodes.size()) && !nodes[child_id].is_deleted) {
                    return true;
                }
            }
        }
        return false;
    }

    /**
     * Usage:
     *     bool can_undo = tree.can_step_backward();
     * Usage Example:
     *     if (tree.can_step_backward()) { ... }
     * Description:
     *     Returns true if active node is not root.
     */
    bool can_step_backward() const {
        return current_node_id > 0;
    }



    /**
     * Usage:
     *     std::string pgn = tree.to_pgn_string(11, 2);
     * Usage Example:
     *     auto pgn = tree.to_pgn_string(11, 1);
     * Description:
     *     Formats entire move tree into standard PGN string including which color went first
     *     and branching variations in parentheses.
     */
    std::string to_pgn_string(int size, uint8_t default_first_player = 2) const {
        uint8_t first_color = default_first_player;
        if (nodes.size() > 1 && !nodes[0].children.empty()) {
            int first_id = nodes[0].children[0];
            if (first_id >= 0 && first_id < static_cast<int>(nodes.size())) {
                first_color = nodes[first_id].player;
            }
        }
        std::stringstream ss;
        ss << "[Game \"Hex\"]\n[Size \"" << size << "x" << size << "\"]\n";
        ss << "[First \"" << (first_color == 1 ? "Red" : "Blue") << "\"]\n\n";
        format_tree_pgn_recursive(0, ss, false);
        return ss.str();
    }

    /**
     * Usage:
     *     tree.load_pgn_tree(pgn_text, 11, starting_player, &out_player, &out_size);
     * Usage Example:
     *     tree.load_pgn_tree(pgn_str, 11, 1, &detected_player, &detected_size);
     * Description:
     *     Parses PGN text including [First "Color"] header, comments, and branching variations in parentheses into the MoveTree.
     */
    void load_pgn_tree(const std::string& pgn_text, int size, uint8_t initial_player = 1, uint8_t* out_initial_player = nullptr, int* out_size = nullptr) {
        clear();
        std::vector<int> branch_stack;
        int curr_id = 0;
        size_t i = 0;
        uint8_t current_first_player = initial_player;
        int parsed_size = size;

        while (i < pgn_text.size()) {
            char ch = pgn_text[i];

            // 1. Skip whitespace
            if (std::isspace(static_cast<unsigned char>(ch))) {
                i++;
                continue;
            }

            // 2. Parse PGN tag pairs: [Tag "Value"]
            if (ch == '[') {
                size_t tag_start = i;
                while (i < pgn_text.size() && pgn_text[i] != ']') i++;
                if (i < pgn_text.size()) i++;
                std::string tag_str = pgn_text.substr(tag_start, i - tag_start);
                std::string key, val;
                if (parse_pgn_tag(tag_str, key, val)) {
                    process_pgn_tag(key, val, current_first_player, parsed_size);
                }
                continue;
            }

            // 3. Skip comments in braces { ... } or semicolons ; ...
            if (ch == '{') {
                while (i < pgn_text.size() && pgn_text[i] != '}') i++;
                if (i < pgn_text.size()) i++;
                continue;
            }
            if (ch == ';') {
                while (i < pgn_text.size() && pgn_text[i] != '\n') i++;
                continue;
            }

            // 4. Open parenthesis: starting a branching variation (dark tree)
            if (ch == '(') {
                if (curr_id > 0 && curr_id < static_cast<int>(nodes.size())) {
                    branch_stack.push_back(nodes[curr_id].parent_id);
                    curr_id = nodes[curr_id].parent_id;
                    current_node_id = curr_id;
                } else {
                    branch_stack.push_back(0);
                    curr_id = 0;
                    current_node_id = 0;
                }
                i++;
                continue;
            }

            // 5. Close parenthesis: ending the branching variation
            if (ch == ')') {
                if (!branch_stack.empty()) {
                    int parent_id = branch_stack.back();
                    branch_stack.pop_back();
                    int resume_id = parent_id;
                    if (parent_id >= 0 && parent_id < static_cast<int>(nodes.size())) {
                        for (int child_id : nodes[parent_id].children) {
                            if (child_id >= 0 && child_id < static_cast<int>(nodes.size()) && !nodes[child_id].is_deleted) {
                                resume_id = child_id;
                                break;
                            }
                        }
                    }
                    curr_id = resume_id;
                    current_node_id = resume_id;
                }
                i++;
                continue;
            }

            // 6. Skip move numbers: e.g. "1.", "1...", "12."
            if (std::isdigit(static_cast<unsigned char>(ch))) {
                size_t num_start = i;
                while (i < pgn_text.size() && std::isdigit(static_cast<unsigned char>(pgn_text[i]))) i++;
                if (i < pgn_text.size() && pgn_text[i] == '.') {
                    while (i < pgn_text.size() && pgn_text[i] == '.') i++;
                    continue;
                } else {
                    i = num_start;
                }
            }

            // 7. Algebraic move: column letter followed by row number (e.g. "f6", "k1", "e10")
            if (std::isalpha(static_cast<unsigned char>(ch))) {
                char col_char = static_cast<char>(std::tolower(static_cast<unsigned char>(ch)));
                int col = col_char - 'a';
                i++;

                std::string row_str;
                while (i < pgn_text.size() && std::isdigit(static_cast<unsigned char>(pgn_text[i]))) {
                    row_str += pgn_text[i];
                    i++;
                }

                if (!row_str.empty()) {
                    int row = std::stoi(row_str) - 1;
                    int board_dim = (parsed_size >= 3 && parsed_size <= 13) ? parsed_size : size;
                    if (row >= 0 && row < board_dim && col >= 0 && col < board_dim) {
                        uint8_t move_player = (nodes[curr_id].ply % 2 == 0) ? current_first_player : (current_first_player == 1 ? 2 : 1);
                        curr_id = add_or_select_move(row, col, move_player);
                    }
                }
                continue;
            }

            i++;
        }

        if (out_initial_player) *out_initial_player = current_first_player;
        if (out_size) *out_size = parsed_size;
    }

    /**
     * Usage:
     *     tree.load_linear_moves(moves, initial_player);
     * Usage Example:
     *     tree.load_linear_moves(parsed_moves, BLUE);
     * Description:
     *     Replaces tree with a linear sequence of moves.
     */
    void load_linear_moves(const std::vector<std::pair<int, int>>& moves, uint8_t first_player = 1) {
        clear();
        uint8_t cur_player = first_player;
        for (const auto& [r, c] : moves) {
            add_or_select_move(r, c, cur_player);
            cur_player = (cur_player == 1) ? 2 : 1;
        }
    }

private:
    static std::string format_node_metadata(const MoveNode& node) {
        if (node.depth <= 0 && node.elapsed_sec <= 0.0 && node.eval_score == 0.0f) {
            return "";
        }
        std::stringstream ss;
        ss << "{";
        bool first = true;
        if (node.depth > 0) {
            ss << "[%depth " << node.depth << "]";
            first = false;
        }
        if (node.elapsed_sec > 0.0) {
            if (!first) ss << " ";
            ss << "[%emt " << std::fixed << std::setprecision(3) << node.elapsed_sec << "]";
            first = false;
        }
        if (node.eval_score != 0.0f) {
            if (!first) ss << " ";
            ss << "[%eval " << std::showpos << std::fixed << std::setprecision(2) << node.eval_score << std::noshowpos << "]";
        }
        ss << "} ";
        return ss.str();
    }

    void format_tree_pgn_recursive(int node_id, std::stringstream& ss, bool start_with_num) const {
        if (node_id < 0 || node_id >= static_cast<int>(nodes.size())) return;
        const auto& node = nodes[node_id];

        std::vector<int> active_children;
        for (int ch_id : node.children) {
            if (ch_id >= 0 && ch_id < static_cast<int>(nodes.size())) {
                if (!nodes[ch_id].is_deleted) {
                    active_children.push_back(ch_id);
                }
            }
        }
        if (active_children.empty()) return;

        int primary_id = active_children[0];
        const auto& prim_node = nodes[primary_id];

        int move_num = (prim_node.ply + 1) / 2;
        std::string prim_meta = format_node_metadata(prim_node);
        if (prim_node.ply % 2 != 0) {
            ss << move_num << ". " << prim_node.to_algebraic() << " " << prim_meta;
        } else {
            if (start_with_num) {
                ss << move_num << "... " << prim_node.to_algebraic() << " " << prim_meta;
            } else {
                ss << prim_node.to_algebraic() << " " << prim_meta;
            }
        }

        // Branching variations (dark move trees) from the same parent enclosed in parentheses
        for (size_t v = 1; v < active_children.size(); v++) {
            int var_id = active_children[v];
            const auto& var_node = nodes[var_id];
            ss << "(";
            int v_num = (var_node.ply + 1) / 2;
            std::string var_meta = format_node_metadata(var_node);
            if (var_node.ply % 2 != 0) {
                ss << v_num << ". " << var_node.to_algebraic() << " " << var_meta;
            } else {
                ss << v_num << "... " << var_node.to_algebraic() << " " << var_meta;
            }
            format_tree_pgn_recursive(var_id, ss, false);
            ss << ") ";
        }

        bool next_needs_num = (active_children.size() > 1);
        format_tree_pgn_recursive(primary_id, ss, next_needs_num);
    }


    bool is_descendant_of(int check_id, int ancestor_id) const {
        int curr = check_id;
        while (curr > 0 && curr < static_cast<int>(nodes.size())) {
            if (curr == ancestor_id) return true;
            curr = nodes[curr].parent_id;
        }
        return false;
    }

    void mark_subtree_deleted(int node_id) {
        if (node_id < 0 || node_id >= static_cast<int>(nodes.size())) return;
        nodes[node_id].is_deleted = true;
        for (int child_id : nodes[node_id].children) {
            mark_subtree_deleted(child_id);
        }
    }
};

} // namespace hex::gui
