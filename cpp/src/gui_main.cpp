/**
 * Standalone C++20 Hex GUI Application Binary.
 *
 * OOP Description:
 * Entry point launching the native hardware-accelerated SDL3 Hex GUI window,
 * managing the interactive diamond board layout, async search, and analysis pane.
 * Default Author: Logan Kirkendall, Logan@LKAud.io
 */

#include "hex_gui.hpp"
#include <iostream>

int main(int argc, char* argv[]) {
    int board_size = 11;
    if (argc > 1) {
        board_size = std::stoi(argv[1]);
    }

    std::cout << "Starting Hex Nash C++ Native GUI (Board Size: " << board_size << "x" << board_size << ")...\n";

    try {
        hex::gui::HexGUIApp app(board_size);
        app.run();
    } catch (const std::exception& e) {
        std::cerr << "GUI Error: " << e.what() << "\n";
        return 1;
    }

    return 0;
}
