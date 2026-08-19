/**
 * Standalone C++20 Hex Solver CLI Application.
 *
 * OOP Description:
 * Command-line entry point providing high-speed headless Hex search benchmarks,
 * position evaluation, and move suggestion using the native C++20 engine wrapper.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include "hex_engine.hpp"
#include <chrono>
#include <iomanip>

int main(int argc, char* argv[]) {
    int size = 11;
    int depth = 14;
    uint8_t player = hex::BLUE;

    for (int i = 1; i < argc; i++) {
        std::string arg = argv[i];
        if ((arg == "--size" || arg == "-s") && i + 1 < argc) {
            size = std::stoi(argv[++i]);
        } else if ((arg == "--depth" || arg == "-d") && i + 1 < argc) {
            depth = std::stoi(argv[++i]);
        } else if ((arg == "--player" || arg == "-p") && i + 1 < argc) {
            std::string p_str = argv[++i];
            player = (p_str[0] == 'R' || p_str[0] == 'r') ? hex::RED : hex::BLUE;
        }
    }

    std::cout << "==================================================\n";
    std::cout << "  Hex Nash C++20 Native Engine\n";
    std::cout << "  Board: " << size << "x" << size << " | Depth: " << depth
              << " | Player: " << (player == hex::RED ? "RED" : "BLUE") << "\n";
    std::cout << "==================================================\n";

    hex::HexBoard board(size);
    hex::HexEngine engine;

    auto t0 = std::chrono::high_resolution_clock::now();
    auto result = engine.search(board, player, depth);
    auto t1 = std::chrono::high_resolution_clock::now();

    double elapsed = std::chrono::duration<double>(t1 - t0).count();
    uint64_t nps = static_cast<uint64_t>(result.nodes / std::max(0.0001, elapsed));

    if (result.best_move.has_value()) {
        auto [r, c] = result.best_move.value();
        char col_char = static_cast<char>('A' + c);
        std::cout << "Best Move: " << col_char << (r + 1) << " (r=" << r << ", c=" << c << ")\n";
    } else {
        std::cout << "Best Move: None\n";
    }

    std::cout << std::fixed << std::setprecision(2);
    std::cout << "Eval Score: " << result.score << "\n";
    std::cout << "Nodes: " << result.nodes << " | NPS: " << nps << " n/s | Time: " << elapsed << "s\n";
    std::cout << "Top Moves Leaderboard:\n";
    for (const auto& tm : result.top_moves) {
        char col_char = static_cast<char>('A' + tm.c);
        std::cout << "  #" << tm.rank << ": " << col_char << (tm.r + 1)
                  << " | Score: " << std::showpos << tm.score << std::noshowpos << "\n";
    }
    std::cout << "==================================================\n";

    return 0;
}
