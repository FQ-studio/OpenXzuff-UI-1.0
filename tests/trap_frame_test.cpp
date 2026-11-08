#include "openxzuff/kernel/riscv64/trap_frame.hpp"

#include <cstdint>
#include <iostream>

int main() {
    using openxzuff::kernel::riscv64::TrapFrame;
    TrapFrame frame{};
    frame.x[openxzuff::kernel::riscv64::register_sp] = 0x8000;
    frame.x[openxzuff::kernel::riscv64::register_a0] = 42;
    frame.sepc = 0x80200000;
    frame.scause = 8;

    if (frame.x[2] != 0x8000 || frame.x[10] != 42 ||
        frame.sepc != 0x80200000 || frame.scause != 8) {
        std::cerr << "FAIL: RV64 trap frame fields\n";
        return 1;
    }
    std::cout << "RV64 trap frame layout tests: PASS\n";
    return 0;
}
