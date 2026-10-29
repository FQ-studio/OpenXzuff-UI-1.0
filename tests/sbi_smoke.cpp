#include "openxzuff/kernel/sbi.hpp"
#include <cstdint>
#include <iostream>

int main() {
#if !defined(__riscv) || __riscv_xlen != 64
    const auto result = openxzuff::kernel::sbi::get_spec_version();
    if (result.error != -2 || result.value != 0) {
        std::cerr << "SBI host guard failed\n";
        return 1;
    }
#endif
    std::cout << "SBI wrapper smoke test: PASS\n";
    return 0;
}
