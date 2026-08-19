/**
 * Standalone C++20 Benchmark Suite for Hex Nash Engine.
 *
 * OOP Description:
 * Benchmarks engine throughput (nodes/second), search latency across multiple board sizes
 * (5x5, 7x7, 11x11), positions (opening, midgame, tactical ladders), and depths.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include "hex_engine.hpp"
#include <chrono>
#include <iomanip>
#include <iostream>
#include <mach/mach.h>

/**
 * Usage:
 *     double ram_mb = get_process_ram_mb();
 * Usage Example:
 *     double ram = get_process_ram_mb();
 * Description:
 *     Retrieves the current process resident memory (RSS) in megabytes on macOS.
 */
double get_process_ram_mb() {
    mach_task_basic_info info;
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), MACH_TASK_BASIC_INFO, (task_info_t)&info, &count) == KERN_SUCCESS) {
        return static_cast<double>(info.resident_size) / (1024.0 * 1024.0);
    }
    return 0.0;
}

/**
 * Usage:
 *     run_benchmark_case("11x11 Midgame", board, player, depth);
 * Usage Example:
 *     run_benchmark_case("11x11 Complex Ladder", board, hex::BLUE, 6);
 * Description:
 *     Executes a search benchmark case and prints formatted throughput, latency, and RAM statistics.
 */
void run_benchmark_case(const std::string& name, const hex::HexBoard& board, uint8_t player, int depth) {
    hex::HexEngine engine;
    engine.clear_cache();

    auto t0 = std::chrono::high_resolution_clock::now();
    auto res = engine.search(board, player, depth);
    auto t1 = std::chrono::high_resolution_clock::now();

    double elapsed = std::chrono::duration<double>(t1 - t0).count();
    uint64_t nps = static_cast<uint64_t>(res.nodes / std::max(0.00001, elapsed));
    double ram_mb = get_process_ram_mb();

    std::string best_str = "-";
    if (res.best_move.has_value()) {
        auto [r, c] = res.best_move.value();
        best_str = std::string(1, static_cast<char>('A' + c)) + std::to_string(r + 1);
    }

    std::cout << "| " << std::left << std::setw(28) << name
              << "| " << std::setw(6) << depth
              << "| " << std::right << std::setw(10) << res.nodes
              << " | " << std::fixed << std::setprecision(4) << std::setw(9) << elapsed << "s"
              << " | " << std::setw(12) << nps
              << " | " << std::fixed << std::setprecision(1) << std::setw(7) << ram_mb << " MB"
              << " | " << std::left << std::setw(6) << best_str << " |\n";
}

int main() {
    std::cout << "\n========================================================================================\n";
    std::cout << "                 HEX NASH ENGINE C++20 BENCHMARK SUITE                                 \n";
    std::cout << "========================================================================================\n";
    std::cout << "| Scenario                    | Depth |   Nodes    |  Latency  |     NPS      |   RAM   | Best   |\n";
    std::cout << "|-----------------------------|-------|------------|-----------|--------------|---------|--------|\n";

    // 1. 5x5 Small Board Solve (Depth 6 & 8)
    hex::HexBoard b5(5);
    run_benchmark_case("5x5 Empty Board Solve", b5, hex::BLUE, 6);
    run_benchmark_case("5x5 Empty Board Deep", b5, hex::BLUE, 8);

    // 2. 7x7 Midgame Board (Depth 6)
    hex::HexBoard b7(7);
    b7.place_move(3, 3, hex::BLUE);
    b7.place_move(3, 4, hex::RED);
    b7.place_move(2, 4, hex::BLUE);
    b7.place_move(4, 2, hex::RED);
    run_benchmark_case("7x7 4-Stone Midgame", b7, hex::BLUE, 6);

    // 3. 11x11 Move 3 Position (1. f6 g6 2. g5 d7)
    hex::HexBoard b11_open(11);
    b11_open.place_move(5, 5, hex::BLUE);
    b11_open.place_move(5, 6, hex::RED);
    b11_open.place_move(4, 6, hex::BLUE);
    b11_open.place_move(6, 3, hex::RED);
    run_benchmark_case("11x11 4-Stone Opening", b11_open, hex::BLUE, 4);
    run_benchmark_case("11x11 4-Stone Opening", b11_open, hex::BLUE, 6);

    // 4. 11x11 Tactical Ladder Position (10 moves)
    hex::HexBoard b11_mid(11);
    b11_mid.place_move(5, 5, hex::BLUE); // f6
    b11_mid.place_move(5, 6, hex::RED);  // g6
    b11_mid.place_move(4, 6, hex::BLUE); // g5
    b11_mid.place_move(6, 3, hex::RED);  // d7
    b11_mid.place_move(3, 7, hex::BLUE); // h4
    b11_mid.place_move(4, 4, hex::RED);  // e5
    b11_mid.place_move(3, 4, hex::BLUE); // e4
    b11_mid.place_move(3, 5, hex::RED);  // f4
    b11_mid.place_move(2, 5, hex::BLUE); // f3
    b11_mid.place_move(2, 6, hex::RED);  // g3
    run_benchmark_case("11x11 10-Stone Midgame", b11_mid, hex::BLUE, 4);
    run_benchmark_case("11x11 10-Stone Midgame", b11_mid, hex::BLUE, 6);

    std::cout << "========================================================================================\n\n";
    return 0;
}
