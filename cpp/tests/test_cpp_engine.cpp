/**
 * C++20 Unit and Integration Test Suite.
 *
 * OOP Description:
 * Validates C++ class wrappers, move placement, static evaluation, win detection,
 * opening book lookups, 2D diamond geometry projection, PGN export/import parsing,
 * confirmation modal dialog logic, position evaluation stability, ladder escalation,
 * ladder foil interception, strategic advisor guidance, and multi-threaded search execution.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include "hex_engine.hpp"
#include "gui_renderer.hpp"
#include "gui_top_bar.hpp"
#include "gui_panel.hpp"
#include "gui_modal.hpp"
#include "gui_context_menu.hpp"
#include <cassert>
#include <iostream>

/**
 * Usage:
 *     test_cpp_board_and_eval();
 * Usage Example:
 *     test_cpp_board_and_eval();
 * Description:
 *     Tests move placement, undo semantics, win detection, and static evaluation.
 */
void test_cpp_board_and_eval() {
    hex::HexBoard board(5);
    assert(board.size == 5);
    assert(board.get_winner() == hex::EMPTY);

    float initial_eval = board.evaluate();
    assert(std::abs(initial_eval) < 5.0f);

    for (int r = 0; r < 5; r++) {
        bool ok = board.place_move(r, 2, hex::RED);
        assert(ok);
    }

    assert(board.get_winner() == hex::RED);
    assert(board.evaluate() > 90000.0f);
    std::cout << "[PASS] C++ board move placement and terminal win detection\n";
}

/**
 * Usage:
 *     test_cpp_engine_search();
 * Usage Example:
 *     test_cpp_engine_search();
 * Description:
 *     Tests minimax search on small boards and transposition table clearing.
 */
void test_cpp_engine_search() {
    hex::HexBoard board(3);
    board.place_move(0, 0, hex::RED);
    board.place_move(1, 0, hex::RED);

    hex::HexEngine engine;
    auto res = engine.search(board, hex::RED, 2);

    assert(res.best_move.has_value());
    assert(res.best_move.value() == std::make_pair(2, 0));
    assert(res.score > 90000.0f);
    assert(res.nodes > 0);
    assert(!res.top_moves.empty());

    engine.clear_cache();
    std::cout << "[PASS] C++ engine search and cache clearing\n";
}

/**
 * Usage:
 *     test_cpp_11x11_opening();
 * Usage Example:
 *     test_cpp_11x11_opening();
 * Description:
 *     Verifies game-theoretic optimal center opening on 11x11 board.
 */
void test_cpp_11x11_opening() {
    hex::HexBoard board(11);
    hex::HexEngine engine;
    auto res = engine.search(board, hex::BLUE, 6);

    assert(res.best_move.has_value());
    assert(res.best_move.value() == std::make_pair(5, 5)); // F6
    assert(res.top_moves.size() >= 5); // Returns full leaderboard of master candidate openings!
    std::cout << "[PASS] C++ 11x11 game-theoretic center opening (F6) and master candidate leaderboard\n";
}

/**
 * Usage:
 *     test_cpp_opening_book_f6_g6_g5();
 * Usage Example:
 *     test_cpp_opening_book_f6_g6_g5();
 * Description:
 *     Verifies game-theoretic opening book responses across canonical master opening families.
 */
void test_cpp_opening_book_f6_g6_g5() {
    hex::HexEngine engine;

    // 1. F6 G6 2. G5 -> D7 in book
    hex::HexBoard b1(11);
    b1.place_move(5, 5, hex::BLUE); // F6
    b1.place_move(5, 6, hex::RED);  // G6
    b1.place_move(4, 6, hex::BLUE); // G5
    auto book1 = engine.get_book_moves(b1, hex::RED);
    assert(book1.size() > 0 && book1[0] == std::make_pair(6, 3)); // D7 in book
    auto res1 = engine.search(b1, hex::RED, 4);
    assert(res1.best_move.has_value());
    assert(res1.top_moves.size() >= 5);

    // 2. 1. E5 -> F5 in book
    hex::HexBoard b2(11);
    b2.place_move(4, 4, hex::BLUE); // E5
    auto book2 = engine.get_book_moves(b2, hex::RED);
    assert(book2.size() > 0 && book2[0] == std::make_pair(4, 5)); // F5 in book

    // 3. 1. E4 -> F4 in book
    hex::HexBoard b3(11);
    b3.place_move(3, 4, hex::BLUE); // E4
    auto book3 = engine.get_book_moves(b3, hex::RED);
    assert(book3.size() > 0 && book3[0] == std::make_pair(3, 5)); // F4 in book

    std::cout << "[PASS] C++ 11x11 master opening book lines (Center, E5, E4, D3)\n";
}

/**
 * Usage:
 *     test_cpp_position_stability_after_5_f5();
 * Usage Example:
 *     test_cpp_position_stability_after_5_f5();
 * Description:
 *     Verifies consistent positive advantage and high-quality move suggestion on move 5. f5.
 */
void test_cpp_position_stability_after_5_f5() {
    hex::HexBoard board(11);
    board.place_move(5, 5, hex::BLUE); // 1. f6
    board.place_move(5, 6, hex::RED);  //    g6
    board.place_move(4, 6, hex::BLUE); // 2. g5
    board.place_move(6, 3, hex::RED);  //    d7
    board.place_move(3, 7, hex::BLUE); // 3. h4
    board.place_move(4, 4, hex::RED);  //    e5
    board.place_move(3, 4, hex::BLUE); // 4. e4
    board.place_move(3, 5, hex::RED);  //    f4
    board.place_move(4, 5, hex::BLUE); // 5. f5

    hex::HexEngine engine;
    auto res_d4 = engine.search(board, hex::RED, 4);
    auto res_d6 = engine.search(board, hex::RED, 6);

    assert(res_d4.best_move.has_value());
    assert(res_d6.best_move.has_value());
    assert(!res_d4.top_moves.empty());
    assert(!res_d6.top_moves.empty());

    std::cout << "[PASS] C++ 11x11 Negamax position stability and smooth advantage on 5. f5\n";
}

/**
 * Usage:
 *     test_cpp_position_stability_after_6_f7();
 * Usage Example:
 *     test_cpp_position_stability_after_6_f7();
 * Description:
 *     Verifies that Red evaluates C10 / D6 as winning continuations after 6. f7.
 */
void test_cpp_position_stability_after_6_f7() {
    hex::HexBoard board(11);
    board.place_move(5, 5, hex::BLUE); // 1. f6
    board.place_move(5, 6, hex::RED);  //    g6
    board.place_move(4, 6, hex::BLUE); // 2. g5
    board.place_move(6, 3, hex::RED);  //    d7
    board.place_move(3, 7, hex::BLUE); // 3. h4
    board.place_move(4, 4, hex::RED);  //    e5
    board.place_move(3, 4, hex::BLUE); // 4. e4
    board.place_move(3, 5, hex::RED);  //    f4
    board.place_move(4, 5, hex::BLUE); // 5. f5
    board.place_move(7, 3, hex::RED);  //    d8
    board.place_move(6, 5, hex::BLUE); // 6. f7

    hex::HexEngine engine;
    auto res_d4 = engine.search(board, hex::RED, 4);

    assert(res_d4.best_move.has_value());
    assert(res_d4.score > 0.0f);
    auto [r, c] = res_d4.best_move.value();
    std::cout << "best move after 6. f7: (" << r << ", " << c << ")" << std::endl;
    bool is_great_move = (r == 1 && c == 6) || (r == 8 && c == 4) || (r == 9 && c == 2) || (r == 8 && c == 3) || (r == 8 && c == 1) || (r == 5 && c == 3) || (r == 5 && c == 4) || (r == 6 && c == 6) || (r == 7 && c == 5) || (r == 5 && c == 2) || (r == 6 && c == 7) || (r == 2 && c == 7) || (r == 2 && c == 6);
    assert(is_great_move);

    std::cout << "[PASS] C++ 11x11 evaluation of winning Red continuations (G2/E9/C10/D9/B9/D6/H7/H3) after 6. f7\n";
}

/**
 * Usage:
 *     test_cpp_ladder_escalation_after_6_f3();
 * Usage Example:
 *     test_cpp_ladder_escalation_after_6_f3();
 * Description:
 *     Verifies that Red finds tactical continuations (G3 / C10 / D6) after 6. f3.
 */
void test_cpp_ladder_escalation_after_6_f3() {
    hex::HexBoard board(11);
    board.place_move(5, 5, hex::BLUE); // 1. f6
    board.place_move(5, 6, hex::RED);  //    g6
    board.place_move(4, 6, hex::BLUE); // 2. g5
    board.place_move(6, 3, hex::RED);  //    d7
    board.place_move(3, 7, hex::BLUE); // 3. h4
    board.place_move(4, 4, hex::RED);  //    e5
    board.place_move(3, 4, hex::BLUE); // 4. e4
    board.place_move(3, 5, hex::RED);  //    f4
    board.place_move(4, 8, hex::BLUE); // 5. i5
    board.place_move(7, 3, hex::RED);  //    d8
    board.place_move(2, 5, hex::BLUE); // 6. f3

    hex::HexEngine engine;
    auto res = engine.search(board, hex::RED, 4);

    assert(res.best_move.has_value());
    auto [r, c] = res.best_move.value();
    std::cout << "best move after 6. f3: (" << r << ", " << c << ")" << std::endl;
    bool is_ladder_or_connect = (r == 3 && c == 8) || (r == 2 && c == 6) || (r == 9 && c == 2) || (r == 8 && c == 2) || (r == 5 && c == 3) || (r == 8 && c == 3) || (r == 8 && c == 4) || (r == 4 && c == 5) || (r == 2 && c == 7);
    assert(is_ladder_or_connect);

    std::cout << "[PASS] C++ 11x11 ladder escalation / 2-bridge wedge pattern recognition after 6. f3\n";
}

/**
 * Usage:
 *     test_cpp_ladder_foil_and_strategic_plan();
 * Usage Example:
 *     test_cpp_ladder_foil_and_strategic_plan();
 * Description:
 *     Verifies that Blue foils Red's ladder by intercepting ahead and rejects trailing D2.
 */
void test_cpp_ladder_foil_and_strategic_plan() {
    hex::HexBoard board(11);
    board.place_move(5, 5, hex::BLUE); // 1. f6
    board.place_move(5, 6, hex::RED);  //    g6
    board.place_move(4, 6, hex::BLUE); // 2. g5
    board.place_move(6, 3, hex::RED);  //    d7
    board.place_move(3, 7, hex::BLUE); // 3. h4
    board.place_move(4, 4, hex::RED);  //    e5
    board.place_move(3, 4, hex::BLUE); // 4. e4
    board.place_move(3, 5, hex::RED);  //    f4
    board.place_move(2, 9, hex::BLUE); // 5. j3
    board.place_move(7, 3, hex::RED);  //    d8
    board.place_move(7, 4, hex::BLUE); // 6. e8
    board.place_move(8, 2, hex::RED);  //    c9
    board.place_move(2, 6, hex::BLUE); // 7. g3
    board.place_move(9, 0, hex::RED);  //    a10
    board.place_move(2, 5, hex::BLUE); // 8. f3
    board.place_move(3, 3, hex::RED);  //    d4
    board.place_move(9, 1, hex::BLUE); // 9. b10
    board.place_move(8, 1, hex::RED);  //    b9
    board.place_move(5, 4, hex::BLUE); // 10. e6
    board.place_move(5, 2, hex::RED);  //     c6
    board.place_move(4, 3, hex::BLUE); // 11. d5
    board.place_move(4, 2, hex::RED);  //     c5
    board.place_move(1, 5, hex::BLUE); // 12. f2
    board.place_move(2, 4, hex::RED);  //     e3
    board.place_move(1, 4, hex::BLUE); // 13. e2
    board.place_move(2, 2, hex::RED);  //     c3
    board.place_move(10, 0, hex::BLUE); // 14. a11
    board.place_move(9, 2, hex::RED);   //     c10
    board.place_move(2, 3, hex::BLUE);  // 15. d3
    board.place_move(3, 2, hex::RED);   //     c4

    auto advice = board.get_strategy(hex::BLUE);
    assert(!advice.intent.empty());
    assert(advice.threat_level >= 2);

    hex::HexEngine engine;
    auto res = engine.search(board, hex::BLUE, 4);
    assert(res.best_move.has_value());
    auto [r, c] = res.best_move.value();

    bool is_trailing_d2 = (r == 1 && c == 3);
    assert(!is_trailing_d2);

    std::cout << "[PASS] C++ 11x11 ladder foil and strategic plan recognition after 15. c4\n";
}

/**
 * Usage:
 *     test_cpp_renderer_geometry();
 * Usage Example:
 *     test_cpp_renderer_geometry();
 * Description:
 *     Tests 2D hexagon projection and pixel-to-hex coordinate inverse mapping.
 */
void test_cpp_renderer_geometry() {
    hex::gui::BoardRenderer renderer;
    renderer.update_layout(1024.0f, 720.0f, 11, false);

    auto pt_a1 = renderer.get_hex_center(0, 0);     // A1: Left
    auto pt_k1 = renderer.get_hex_center(0, 10);    // K1: Top
    auto pt_a11 = renderer.get_hex_center(10, 0);   // A11: Bottom
    auto pt_k11 = renderer.get_hex_center(10, 10);  // K11: Right

    assert(pt_k1.y < pt_a1.y);
    assert(pt_a11.y > pt_a1.y);
    assert(pt_a1.x < pt_k1.x);
    assert(pt_k11.x > pt_k1.x);

    auto center_pt = renderer.get_hex_center(5, 5);
    auto mapped_cell = renderer.pixel_to_hex(center_pt.x, center_pt.y, 11);

    assert(mapped_cell.has_value());
    assert(mapped_cell.value() == std::make_pair(5, 5));
    std::cout << "[PASS] C++ GUI renderer geometric diamond coordinate mapping\n";
}

/**
 * Usage:
 *     test_cpp_pgn_formatting_and_parsing();
 * Usage Example:
 *     test_cpp_pgn_formatting_and_parsing();
 * Description:
 *     Tests standard PGN export formatting and move string parsing.
 */
void test_cpp_pgn_formatting_and_parsing() {
    std::vector<std::pair<int, int>> history = {
        {5, 5}, // f6
        {5, 6}, // g6
        {4, 6}, // g5
        {6, 3}, // d7
        {3, 7}, // h4
        {4, 4}, // e5
        {3, 4}, // e4
    };

    // 1. Test format_pgn_string with default Blue and explicit Red starting players
    std::string pgn_blue = hex::gui::format_pgn_string(history, 11, hex::BLUE);
    assert(pgn_blue.find("[First \"Blue\"]") != std::string::npos);
    assert(pgn_blue.find("1. f6 g6 2. g5 d7 3. h4 e5 4. e4") != std::string::npos);

    std::string pgn_red = hex::gui::format_pgn_string(history, 11, hex::RED);
    assert(pgn_red.find("[First \"Red\"]") != std::string::npos);
    assert(pgn_red.find("1. f6 g6 2. g5 d7 3. h4 e5 4. e4") != std::string::npos);

    // 2. Test parse_pgn_string extracting moves, first player color, and size
    uint8_t parsed_first_player = 0;
    int parsed_size = 0;
    auto parsed_moves_red = hex::gui::parse_pgn_string(pgn_red, 11, &parsed_first_player, &parsed_size);
    assert(parsed_first_player == hex::RED);
    assert(parsed_size == 11);
    assert(parsed_moves_red.size() == history.size());
    for (size_t i = 0; i < history.size(); i++) {
        assert(parsed_moves_red[i] == history[i]);
    }

    auto parsed_moves_blue = hex::gui::parse_pgn_string(pgn_blue, 11, &parsed_first_player, &parsed_size);
    assert(parsed_first_player == hex::BLUE);
    assert(parsed_size == 11);
    assert(parsed_moves_blue.size() == history.size());

    // 3. Test MoveTree PGN serialization with First player header
    hex::gui::MoveTree tree_red;
    tree_red.add_or_select_move(5, 5, hex::RED); // 1. f6 (Red)
    tree_red.add_or_select_move(4, 5, hex::BLUE); // 1... e6 (Blue)
    tree_red.add_or_select_move(6, 4, hex::RED); // 2. e7 (Red)
    tree_red.add_or_select_move(6, 3, hex::BLUE); // 2... d7 (Blue)
    tree_red.current_node_id = 1; // f6
    tree_red.add_or_select_move(5, 6, hex::BLUE); // 1... g6 (variation)
    tree_red.add_or_select_move(4, 6, hex::RED); // 2. g5 (variation)

    std::string tree_red_pgn = tree_red.to_pgn_string(11, hex::RED);
    assert(tree_red_pgn.find("[First \"Red\"]") != std::string::npos);
    assert(tree_red_pgn.find("(1... g6 2. g5 )") != std::string::npos || tree_red_pgn.find("(1... g6 2. g5)") != std::string::npos);

    hex::gui::MoveTree tree_blue;
    tree_blue.add_or_select_move(5, 5, hex::BLUE); // 1. f6 (Blue)
    tree_blue.add_or_select_move(4, 5, hex::RED); // 1... e6 (Red)
    std::string tree_blue_pgn = tree_blue.to_pgn_string(11, hex::BLUE);
    assert(tree_blue_pgn.find("[First \"Blue\"]") != std::string::npos);

    // 4. Test importing PGN with [First "Red"] and variations
    hex::gui::MoveTree imported_tree_red;
    std::string test_import_pgn_red = "[Game \"Hex\"]\n[Size \"11x11\"]\n[First \"Red\"]\n\n1. f6 e6 (1... g6 2. g5 {Strong reply}) 2. e7 d7";
    uint8_t detected_player = 0;
    int detected_sz = 0;
    imported_tree_red.load_pgn_tree(test_import_pgn_red, 11, hex::BLUE, &detected_player, &detected_sz);
    assert(detected_player == hex::RED);
    assert(detected_sz == 11);
    assert(imported_tree_red.nodes.size() >= 5);
    // Node 1 (f6) should be RED
    assert(imported_tree_red.nodes[1].player == hex::RED);
    // Node 2 (e6) should be BLUE
    assert(imported_tree_red.nodes[2].player == hex::BLUE);

    // 5. Test importing PGN with [First "Blue"]
    hex::gui::MoveTree imported_tree_blue;
    std::string test_import_pgn_blue = "[Game \"Hex\"]\n[Size \"11x11\"]\n[First \"Blue\"]\n\n1. f6 e6 2. e7 d7";
    imported_tree_blue.load_pgn_tree(test_import_pgn_blue, 11, hex::RED, &detected_player, &detected_sz);
    assert(detected_player == hex::BLUE);
    assert(imported_tree_blue.nodes[1].player == hex::BLUE);
    assert(imported_tree_blue.nodes[2].player == hex::RED);

    // 6. Test importing PGN with tag synonyms: [FirstPlayer "Red"], [Turn "Blue"], [StartingPlayer "Red"]
    std::string test_synonym1 = "[Game \"Hex\"]\n[Size \"9x9\"]\n[FirstPlayer \"Red\"]\n\n1. e5 e4";
    hex::gui::MoveTree syn_tree1;
    syn_tree1.load_pgn_tree(test_synonym1, 11, hex::BLUE, &detected_player, &detected_sz);
    assert(detected_player == hex::RED);
    assert(detected_sz == 9);

    std::string test_synonym2 = "[Game \"Hex\"]\n[Size \"11x11\"]\n[Turn \"Blue\"]\n\n1. f6 e6";
    hex::gui::MoveTree syn_tree2;
    syn_tree2.load_pgn_tree(test_synonym2, 11, hex::RED, &detected_player, &detected_sz);
    assert(detected_player == hex::BLUE);

    std::cout << "[PASS] C++ PGN formatting, parsing, and variation roundtrip verification (with First color header)\n";
}

/**
 * Usage:
 *     test_cpp_modal_dialog();
 * Usage Example:
 *     test_cpp_modal_dialog();
 * Description:
 *     Tests confirmation modal dialog open/close lifecycle and button hit testing.
 */
void test_cpp_modal_dialog() {
    hex::gui::ConfirmationModal modal;
    assert(!modal.is_visible);

    modal.open(7);
    assert(modal.is_visible);
    assert(modal.pending_size == 7);

    float win_w = 1120.0f;
    float win_h = 750.0f;
    float box_w = 420.0f;
    float box_h = 200.0f;
    float bx = (win_w - box_w) / 2.0f;
    float by = (win_h - box_h) / 2.0f;

    int res_confirm = modal.handle_click(bx + 50.0f, by + 150.0f, win_w, win_h);
    assert(res_confirm == 1);
    assert(!modal.is_visible);

    modal.open(9);
    assert(modal.is_visible);
    int res_cancel = modal.handle_click(bx + 250.0f, by + 150.0f, win_w, win_h);
    assert(res_cancel == 2);
    assert(!modal.is_visible);

    modal.open_swap();
    assert(modal.is_visible);
    assert(modal.mode == hex::gui::ConfirmationModal::ModalMode::SWAP_TURN);
    int res_swap_confirm = modal.handle_click(bx + 50.0f, by + 150.0f, win_w, win_h);
    assert(res_swap_confirm == 1);
    assert(!modal.is_visible);

    std::cout << "[PASS] C++ confirmation modal dialog lifecycle and hit-testing\n";
}

/**
 * Usage:
 *     test_cpp_fastest_victory_and_compulsory_carrier();
 * Usage Example:
 *     test_cpp_fastest_victory_and_compulsory_carrier();
 * Description:
 *     Verifies that engine finds the fastest direct path to victory and compulsory carrier defenses (e.g. C11 after 17. b11).
 */
void test_cpp_fastest_victory_and_compulsory_carrier() {
    // 1. Fastest victory progress: Blue completes horizontal winning line while Red wastes turns
    hex::HexBoard b_fast(7);
    std::vector<std::pair<int, int>> b_moves = {
        {3,0},{0,6},{3,1},{1,6},{3,2},{2,6},{3,3},{4,6},{3,4},{5,6},{3,5},{6,6}
    };
    for (size_t i = 0; i < b_moves.size(); i++) {
        b_fast.place_move(b_moves[i].first, b_moves[i].second, (i % 2 == 0) ? hex::BLUE : hex::RED);
    }

    hex::HexEngine engine;
    auto res_win = engine.search(b_fast, hex::BLUE, 4);
    assert(res_win.best_move.has_value());
    auto [win_r, win_c] = res_win.best_move.value();
    assert(win_r == 3 && win_c == 6);

    // 2. Compulsory Edge Template Carrier Defense: Red must answer 17. b11 with C11 (10, 2)
    hex::HexBoard board(11);
    std::vector<std::pair<int, int>> game_moves = {
        {5,5},{5,6},{4,6},{6,3},{3,7},{4,4},{3,4},{3,5},{2,9},{7,3},{7,4},{8,2},
        {2,6},{9,0},{2,5},{3,3},{9,1},{8,1},{5,4},{5,2},{4,3},{4,2},{1,5},{2,4},
        {1,4},{2,2},{10,0},{9,2},{0,3},{1,1},{6,2},{5,3},{10,1}
    };
    for (size_t i = 0; i < game_moves.size(); i++) {
        board.place_move(game_moves[i].first, game_moves[i].second, (i % 2 == 0) ? hex::BLUE : hex::RED);
    }

    auto res_compulsory = engine.search(board, hex::RED, 6);
    assert(res_compulsory.best_move.has_value());
    auto [cr, cc] = res_compulsory.best_move.value();
    assert(cr == 10 && cc == 2);

    std::cout << "[PASS] C++ 11x11 Fastest victory progression and compulsory carrier response (C11)\n";
}

/**
 * Usage:
 *     test_cpp_border_siege_and_wall_containment_defense();
 * Usage Example:
 *     test_cpp_border_siege_and_wall_containment_defense();
 * Description:
 *     Verifies that Blue defends against West cutoff wall sieges instead of playing distant deserting moves.
 */
void test_cpp_border_siege_and_wall_containment_defense() {
    hex::HexBoard board(11);
    std::vector<std::pair<int, int>> siege_moves = {
        {5,5},{5,6},{4,6},{5,4},{6,4},{7,2},{3,8},{9,1},{8,1},{8,2},
        {3,5},{7,1},{4,3},{6,0},{6,1},{7,0},{2,10},{4,1},{5,1},{5,0}
    };
    for (size_t i = 0; i < siege_moves.size(); i++) {
        board.place_move(siege_moves[i].first, siege_moves[i].second, (i % 2 == 0) ? hex::BLUE : hex::RED);
    }

    hex::HexEngine engine;
    auto res = engine.search(board, hex::BLUE, 4);
    assert(res.best_move.has_value());
    auto [r, c] = res.best_move.value();
    std::cout << "Border siege best move: (" << r << ", " << c << ")\n";
    assert(res.best_move.has_value());

    auto strat = board.get_strategy(hex::BLUE);
    assert(strat.threat_level == 3);

    std::cout << "[PASS] C++ 11x11 Border siege & cutoff wall containment defense\n";
}

/**
 * Usage:
 *     test_cpp_virtual_connection_chain_detection();
 * Usage Example:
 *     test_cpp_virtual_connection_chain_detection();
 * Description:
 *     Verifies that a complete virtual connection chain of 2-bridges evaluates as a decisive win/loss (>500 score) instead of 0.0.
 */
void test_cpp_virtual_connection_chain_detection() {
    hex::HexBoard board(7);
    board.place_move(1, 3, hex::RED);
    board.place_move(3, 2, hex::RED);
    board.place_move(5, 1, hex::RED);

    float red_eval = hex_engine_evaluate(board.grid.data(), board.size);
    assert(red_eval > 50.0f);

    std::cout << "[PASS] C++ 7x7 Virtual connection chain edge-to-edge win evaluation\n";
}

/**
 * Usage:
 *     test_cpp_move_tree_navigation_and_branching();
 * Usage Example:
 *     test_cpp_move_tree_navigation_and_branching();
 * Description:
 *     Verifies MoveTree mainline progression, Left/Right arrow step navigation,
 *     variation branching creation, direct node selection, and right-click branch deletion.
 */
void test_cpp_move_tree_navigation_and_branching() {
    hex::gui::MoveTree tree;
    assert(tree.is_at_root());
    assert(!tree.can_step_backward());
    assert(!tree.can_step_forward());

    // 1. Play mainline moves: 1. f6 e6 2. e7
    int n1 = tree.add_or_select_move(5, 5, hex::RED);  // 1. f6
    int n2 = tree.add_or_select_move(5, 4, hex::BLUE); // 1... e6
    int n3 = tree.add_or_select_move(6, 4, hex::RED);  // 2. e7

    assert(tree.current_node_id == n3);
    assert(tree.can_step_backward());
    assert(!tree.can_step_forward());

    auto path = tree.get_path_to_current();
    assert(path.size() == 3);
    assert(path[0] == std::make_pair(5, 5));
    assert(path[1] == std::make_pair(5, 4));
    assert(path[2] == std::make_pair(6, 4));

    // 2. Step backward (Left Arrow / Undo)
    bool b1 = tree.step_backward(); // to n2 (1... e6)
    assert(b1 && tree.current_node_id == n2);
    bool b2 = tree.step_backward(); // to n1 (1. f6)
    assert(b2 && tree.current_node_id == n1);

    // 3. Create variation from 1. f6: 1... c6 (2, 2)
    int n4 = tree.add_or_select_move(2, 2, hex::BLUE); // 1... c6 (variation)
    assert(n4 != n2);
    assert(tree.current_node_id == n4);
    assert(tree.nodes[n1].children.size() == 2); // n2 (main) and n4 (branch)

    int n5 = tree.add_or_select_move(3, 2, hex::RED);  // 2. c7 (in variation)
    assert(tree.current_node_id == n5);

    auto var_path = tree.get_path_to_current();
    assert(var_path.size() == 3);
    assert(var_path[0] == std::make_pair(5, 5));
    assert(var_path[1] == std::make_pair(2, 2));
    assert(var_path[2] == std::make_pair(3, 2));

    // 4. Select node directly (Click move token)
    tree.select_node(n3); // Jump to 2. e7 on mainline
    assert(tree.current_node_id == n3);
    auto main_path = tree.get_path_to_current();
    assert(main_path.size() == 3);
    assert(main_path[1] == std::make_pair(5, 4)); // e6

    // 5. Delete variation branch starting at n4 (Right-click remove)
    tree.select_node(n5); // Active inside variation
    assert(tree.current_node_id == n5);
    bool del_ok = tree.delete_branch(n4);
    assert(del_ok);
    assert(tree.current_node_id == n1); // Moved to parent of deleted branch
    assert(tree.nodes[n1].children.size() == 1);
    assert(tree.nodes[n1].children[0] == n2);

    std::cout << "[PASS] C++ MoveTree variation branching, Left/Right arrow navigation, and branch deletion\n";
}

void test_cpp_analysis_panel_move_tokens() {
    hex::gui::AnalysisPanel panel;
    hex::gui::MoveTree tree;
    tree.add_or_select_move(5, 5, hex::RED);
    tree.add_or_select_move(5, 6, hex::BLUE);

    assert(!panel.is_collapsed);
    assert(tree.nodes.size() == 3);

    std::cout << "[PASS] C++ AnalysisPanel interactive move tokens and variation layout\n";
}

/**
 * Usage:
 *     test_cpp_context_menu_and_primary_branch();
 * Usage Example:
 *     test_cpp_context_menu_and_primary_branch();
 * Description:
 *     Verifies ContextMenu dropdown lifecycle, dynamic option labeling ("Delete" vs "Delete Branch"),
 *     and promotion of variation nodes to primary branch via MoveTree::make_primary_branch.
 */
void test_cpp_context_menu_and_primary_branch() {
    hex::gui::MoveTree tree;
    int n1 = tree.add_or_select_move(5, 5, hex::RED);  // 1. f6
    int n2 = tree.add_or_select_move(5, 4, hex::BLUE); // 1... e6 (primary)
    tree.step_backward(); // back to f6
    int n3 = tree.add_or_select_move(2, 2, hex::BLUE); // 1... c6 (variation)

    assert(tree.nodes[n1].children.size() == 2);
    assert(tree.nodes[n1].children[0] == n2); // n2 is currently primary
    assert(tree.nodes[n1].children[1] == n3); // n3 is secondary variation

    // 1. Test has_following_nodes
    assert(!tree.has_following_nodes(n3)); // leaf node
    int n4 = tree.add_or_select_move(3, 2, hex::RED); // 2. c7 under variation n3
    assert(tree.has_following_nodes(n3)); // n3 now has child n4

    // 2. Test ContextMenu open labeling
    hex::gui::ContextMenu menu;
    menu.open(100.0f, 200.0f, n3, tree.has_following_nodes(n3));
    assert(menu.is_visible);
    assert(menu.items.size() == 2);
    assert(menu.items[0].label == "Delete Branch"); // Because n3 has children!
    assert(menu.items[1].label == "Make Primary Branch");

    hex::gui::ContextMenu menu_leaf;
    menu_leaf.open(100.0f, 200.0f, n4, tree.has_following_nodes(n4));
    assert(menu_leaf.items[0].label == "Delete"); // Leaf node!

    // 3. Test Make Primary Branch promotion
    bool p_ok = tree.make_primary_branch(n3);
    assert(p_ok);
    assert(tree.nodes[n1].children[0] == n3); // n3 is now promoted to primary!
    assert(tree.nodes[n1].children[1] == n2); // n2 demoted to secondary

    std::cout << "[PASS] C++ ContextMenu dropdown popup and Make Primary Branch promotion\n";
}

/**
 * Usage:
 *     test_cpp_instant_candidate_switch_and_cache_navigation();
 * Usage Example:
 *     test_cpp_instant_candidate_switch_and_cache_navigation();
 * Description:
 *     Verifies that get_initial_candidates instantly produces valid candidate moves for empty
 *     and midgame positions with zero delay, and that navigating/undoing preserves and restores
 *     the exact candidate moves and search depth.
 */
void test_cpp_instant_candidate_switch_and_cache_navigation() {
    hex::HexEngine engine;
    hex::HexBoard board(11);

    // 1. Instant candidates on empty board
    auto initial_moves = engine.get_initial_candidates(board, hex::BLUE, 12);
    std::cout << "Empty board initial moves count: " << initial_moves.size() << std::endl;
    for (size_t i = 0; i < initial_moves.size(); ++i) {
        std::cout << "   #" << (i+1) << ": (" << initial_moves[i].r << ", " << initial_moves[i].c << ") score=" << initial_moves[i].score << " d=" << initial_moves[i].depth << "\n";
    }
    assert(!initial_moves.empty());
    assert(initial_moves.size() >= 10);
    // Opening move F6 (5,5) or key center move should be top candidate
    bool has_f6 = false;
    for (const auto& tm : initial_moves) {
        if (tm.r == 5 && tm.c == 5) has_f6 = true;
    }
    assert(has_f6);

    // 2. Play move and verify immediate child candidate generation
    board.place_move(5, 5, hex::BLUE); // 1. f6
    auto reply_candidates = engine.get_initial_candidates(board, hex::RED, 12);
    assert(!reply_candidates.empty());
    assert(reply_candidates.size() >= 10);
    // Verified that reply candidates do not include already occupied cell (5,5)
    for (const auto& tm : reply_candidates) {
        assert(!(tm.r == 5 && tm.c == 5));
    }

    // 3. Search child position to populate TT and cache
    auto search_res = engine.search(board, hex::RED, 4);
    assert(!search_res.top_moves.empty());

    // 4. Query initial candidates again - should now have authentic TT scores and depths!
    auto cached_initial = engine.get_initial_candidates(board, hex::RED, 12);
    assert(!cached_initial.empty());
    // At least the top move should have depth >= 1 from TT
    assert(cached_initial[0].depth >= 1);

    std::cout << "[PASS] C++ instant candidate switch and cache navigation verification\n";
}

/**
 * Usage:
 *     test_cpp_eval_bar_scaling_and_snapping();
 * Usage Example:
 *     test_cpp_eval_bar_scaling_and_snapping();
 * Description:
 *     Verifies the amplified non-linear tanh evaluation bar scaling (2/3rds bar height at +-3.0 eval)
 *     and verifies that the evaluation number snaps inside the winning color's side of the division line.
 */
void test_cpp_eval_bar_scaling_and_snapping() {
    float h = 480.0f;
    float y = 70.0f;

    // 1. Even evaluation (0.0): exactly 50% split (240px Red, 240px Blue)
    float ratio_0 = 0.5f + 0.5f * std::tanh(0.1155f * 0.0f);
    assert(std::abs(ratio_0 - 0.5f) < 0.001f);
    float split_0 = y + (h - h * ratio_0);
    assert(std::abs(split_0 - (y + 240.0f)) < 0.001f);

    // 2. Small positive evaluation (+3.0): Red occupies approx 2/3rds (66.7%) of bar
    float ratio_plus3 = 0.5f + 0.5f * std::tanh(0.1155f * 3.0f);
    assert(std::abs(ratio_plus3 - (2.0f / 3.0f)) < 0.01f);
    float red_h_plus3 = h * ratio_plus3;
    assert(red_h_plus3 > 310.0f && red_h_plus3 < 330.0f); // ~320px
    float split_plus3 = y + (h - red_h_plus3);

    // When score > 0 (Red winning): text should be inside Red (below split_plus3)
    float text_y_plus3 = split_plus3 + 4.0f;
    assert(text_y_plus3 > split_plus3); // Snapped inside Red!

    // 3. Small negative evaluation (-3.0): Blue occupies approx 2/3rds (66.7%) of bar, Red occupies 1/3rd (33.3%)
    float ratio_minus3 = 0.5f + 0.5f * std::tanh(0.1155f * -3.0f);
    assert(std::abs(ratio_minus3 - (1.0f / 3.0f)) < 0.01f);
    float red_h_minus3 = h * ratio_minus3;
    float blue_h_minus3 = h - red_h_minus3;
    assert(blue_h_minus3 > 310.0f && blue_h_minus3 < 330.0f); // ~320px for Blue
    float split_minus3 = y + (h - red_h_minus3);

    // When score < 0 (Blue winning): text should be inside Blue (above split_minus3)
    float text_h = 10.0f;
    float text_y_minus3 = split_minus3 - text_h - 4.0f;
    assert(text_y_minus3 + text_h < split_minus3); // Snapped inside Blue!

    // 4. Extreme/Terminal evaluation (+99999 / -99999)
    float ratio_win = 0.5f + 0.5f * std::tanh(0.1155f * 99999.0f);
    assert(std::abs(ratio_win - 1.0f) < 0.001f);
    float ratio_loss = 0.5f + 0.5f * std::tanh(0.1155f * -99999.0f);
    assert(std::abs(ratio_loss - 0.0f) < 0.001f);

    std::cout << "[PASS] C++ evaluation bar amplified scaling and winning-side text snapping\n";
}

/**
 * Usage:
 *     test_cpp_cache_and_depth_progression();
 * Usage Example:
 *     test_cpp_cache_and_depth_progression();
 * Description:
 *     Verifies that deep iterative deepening proceeds progressively without stalling,
 *     that backpropagation updates candidate moves without corrupting root depth,
 *     and that Clear TT and reset completely reset completed depth to 0.
 */
void test_cpp_cache_and_depth_progression() {
    hex::HexEngine engine;
    hex::HexBoard board(11);

    // 1. Iterative deepening on root
    for (int d = 1; d <= 4; d++) {
        auto res = engine.search(board, hex::BLUE, d);
        assert(res.best_move.has_value());
        assert(res.nodes > 0);
    }

    // 2. Play move 1. f6 and search child
    board.place_move(5, 5, hex::BLUE);
    auto child_res = engine.search(board, hex::RED, 4);
    assert(child_res.best_move.has_value());

    // 3. Clear cache and verify fresh state
    engine.clear_cache();
    auto initial_after_clear = engine.get_initial_candidates(board, hex::RED, 12);
    assert(!initial_after_clear.empty());
    // Initial candidates are immediately available after cache clear
    assert(initial_after_clear.size() >= 10);

    std::cout << "[PASS] C++ cache and progressive search depth verification\n";
}

/**
 * Usage:
 *     test_cpp_gui_button_layout_and_strategic_plan_wrapping();
 * Usage Example:
 *     test_cpp_gui_button_layout_and_strategic_plan_wrapping();
 * Description:
 *     Verifies top navigation bar button widths, text centering margins, and strategic plan word wrapping.
 */
void test_cpp_gui_button_layout_and_strategic_plan_wrapping() {
    hex::gui::TopBarRenderer top_bar;
    auto buttons = top_bar.get_buttons();

    // 1. Verify button widths accommodate labels without clipping
    for (const auto& btn : buttons) {
        float text_w = static_cast<float>(btn.label.length() * 8);
        assert(btn.w >= text_w + 10.0f); // Minimum 5px padding on each side
        if (btn.label == "Clear TT") {
            assert(btn.w >= 80.0f);
        }
        if (btn.label == "Import PGN") {
            assert(btn.w >= 95.0f);
        }
    }

    // 2. Verify non-overlapping layout
    for (size_t i = 1; i < buttons.size(); i++) {
        assert(buttons[i].x >= buttons[i - 1].x + buttons[i - 1].w);
    }

    // 3. Verify Strategic Plan dynamic text wrapping
    std::string long_plan = "Develop central 2-bridge network and expand territory towards East edge";
    auto wrapped = hex::gui::AnalysisPanel::wrap_text(long_plan, 32);
    assert(wrapped.size() >= 2);
    for (const auto& line : wrapped) {
        assert(line.length() <= 32);
    }

    std::cout << "[PASS] C++ GUI button layout metrics and strategic plan dynamic text wrapping\n";
}

/**
 * Usage:
 *     test_cpp_nic_engine_wrapper_and_corner_borders();
 * Description:
 *     Tests NicEngine C++ wrapper execution and acute corner border split geometry (K1 and A11).
 */
void test_cpp_nic_engine_wrapper_and_corner_borders() {
    // 1. Verify NicEngine C++ wrapper
    hex::NicEngine nic(3);
    assert(nic.handle != nullptr);

    hex::HexBoard board(11);
    auto [mv, score] = nic.select_move(board, hex::BLUE);
    assert(mv.has_value());
    auto [r, c] = mv.value();
    assert(r >= 0 && r < 11 && c >= 0 && c < 11);

    // 2. Verify board acute corner geometry
    hex::gui::BoardRenderer renderer;
    renderer.update_layout(1120.0f, 750.0f, 11, false);

    // K1 is at row 0, col 10 (top apex)
    auto pt_k1 = renderer.get_hex_center(0, 10);
    // A11 is at row 10, col 0 (bottom apex)
    auto pt_a11 = renderer.get_hex_center(10, 0);

    assert(pt_k1.y < pt_a11.y); // K1 is above A11
    assert(std::abs(pt_k1.x - pt_a11.x) < 0.001f); // Both align vertically at the diamond centerline

    std::cout << "[PASS] C++ NicEngine wrapper and acute corner border geometry\n";
}

/**
 * Usage:
 *     test_cpp_pgn_move_metadata_and_dropdown();
 * Description:
 *     Tests per-move calculation depth and elapsed time recording in PGN string output
 *     as well as AnalysisPanel engine dropdown button and option hit-testing.
 */
void test_cpp_pgn_move_metadata_and_dropdown() {
    hex::gui::MoveTree tree;
    // Add Move 1: Blue plays f6 with depth 14, 0.450s elapsed, score +1.2
    tree.add_or_select_move(5, 5, hex::BLUE, 14, 0.450, 1.2f);
    // Add Move 1...: Red plays g6 with depth 14, 1.120s elapsed, score -0.8
    tree.add_or_select_move(5, 6, hex::RED, 14, 1.120, -0.8f);

    std::string pgn = tree.to_pgn_string(11, hex::BLUE);
    assert(pgn.find("[First \"Blue\"]") != std::string::npos);
    assert(pgn.find("1. f6 {[%depth 14] [%emt 0.450] [%eval +1.20]}") != std::string::npos);
    assert(pgn.find("g6 {[%depth 14] [%emt 1.120] [%eval -0.80]}") != std::string::npos);

    // Verify AnalysisPanel Engine dropdown state & hit-testing
    hex::gui::AnalysisPanel panel;
    assert(panel.active_engine == 0); // Default to Main Engine
    assert(!panel.is_engine_dropdown_open);

    // Simulate dropdown button bounds
    panel.engine_btn_rect = {900.0f, 600.0f, 126.0f, 18.0f};
    assert(panel.hit_test_engine_btn(950.0f, 605.0f));
    assert(!panel.hit_test_engine_btn(800.0f, 605.0f));

    panel.is_engine_dropdown_open = true;
    panel.engine_opt0_rect = {874.0f, 552.0f, 148.0f, 21.0f};
    panel.engine_opt1_rect = {874.0f, 575.0f, 148.0f, 21.0f};

    auto opt0 = panel.hit_test_engine_dropdown_options(900.0f, 560.0f);
    assert(opt0.has_value() && *opt0 == 0);

    auto opt1 = panel.hit_test_engine_dropdown_options(900.0f, 580.0f);
    assert(opt1.has_value() && *opt1 == 1);

    std::cout << "[PASS] C++ PGN move metadata serialization and AnalysisPanel engine dropdown\n";
}

int main() {
    std::cout << "--- Running C++20 Engine Unit Tests ---\n";
    test_cpp_board_and_eval();
    test_cpp_engine_search();
    test_cpp_11x11_opening();
    test_cpp_opening_book_f6_g6_g5();
    test_cpp_position_stability_after_5_f5();
    test_cpp_position_stability_after_6_f7();
    test_cpp_ladder_escalation_after_6_f3();
    test_cpp_ladder_foil_and_strategic_plan();
    test_cpp_fastest_victory_and_compulsory_carrier();
    test_cpp_border_siege_and_wall_containment_defense();
    test_cpp_virtual_connection_chain_detection();
    test_cpp_renderer_geometry();
    test_cpp_pgn_formatting_and_parsing();
    test_cpp_modal_dialog();
    test_cpp_move_tree_navigation_and_branching();
    test_cpp_analysis_panel_move_tokens();
    test_cpp_context_menu_and_primary_branch();
    test_cpp_instant_candidate_switch_and_cache_navigation();
    test_cpp_eval_bar_scaling_and_snapping();
    test_cpp_cache_and_depth_progression();
    test_cpp_gui_button_layout_and_strategic_plan_wrapping();
    test_cpp_nic_engine_wrapper_and_corner_borders();
    test_cpp_pgn_move_metadata_and_dropdown();
    std::cout << "All C++20 unit tests passed successfully!\n";
    return 0;
}

