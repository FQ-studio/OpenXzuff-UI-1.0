#include "openxzuff/kernel/xexe_loader.hpp"
#include <array>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <vector>

namespace {
void put(std::vector<std::uint8_t>& bytes, std::size_t offset, std::uint64_t value, std::size_t width) {
    for (std::size_t i = 0; i < width; ++i) bytes[offset + i] = static_cast<std::uint8_t>((value >> (8U * i)) & 0xFFU);
}
std::vector<std::uint8_t> validFile() {
    std::vector<std::uint8_t> file(36, 0);
    const std::array<std::uint8_t, 8> magic{'X','Z','X','E','X','E','1',0};
    for (std::size_t i = 0; i < magic.size(); ++i) file[i] = magic[i];
    put(file, 8, 1, 2); put(file, 10, 32, 2); put(file, 12, 1, 2); put(file, 14, 1, 2);
    put(file, 16, 32, 8); put(file, 24, 4, 4);
    file[32] = 0x13; file[33] = 0x05; file[34] = 0x00; file[35] = 0x00;
    return file;
}
bool check(bool condition, const char* label) {
    if (!condition) std::cerr << "FAIL: " << label << '\n';
    return condition;
}
}

int main(int argc, char** argv) {
    using openxzuff::kernel::XexeValidationError;
    using openxzuff::kernel::XexeView;
    using openxzuff::kernel::validate_xexe;
    bool ok = true;
    auto file = validFile();
    if (argc == 2) {
        std::ifstream input(argv[1], std::ios::binary);
        if (!input) {
            std::cerr << "FAIL: cannot open .xexe input file\n";
            return 1;
        }
        file.assign(std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>());
    } else if (argc != 1) {
        std::cerr << "Usage: xexe-loader-test [program.xexe]\n";
        return 2;
    }
    XexeView view{};
    ok &= check(validate_xexe(file, view) == XexeValidationError::None, "valid image accepted");
    ok &= check(!view.code.empty() && view.entry_offset == 32, "code view and entry offset");
    if (argc == 2) {
        const std::array<std::uint8_t, 12> expectedReturn42{
            0x13, 0x05, 0xA0, 0x02, // addi a0, zero, 42
            0x93, 0x08, 0x00, 0x00, // addi a7, zero, Exit syscall 0
            0x73, 0x00, 0x00, 0x00  // ecall
        };
        ok &= check(view.code.size() == expectedReturn42.size(), "return_42 machine-code size");
        if (view.code.size() == expectedReturn42.size()) {
            for (std::size_t i = 0; i < expectedReturn42.size(); ++i) {
                ok &= check(view.code[i] == expectedReturn42[i], "return_42 RV64 instruction bytes");
            }
        }
        if (ok) std::cout << "Compiler-produced return_42.xexe validation and machine-code test: PASS\n";
        return ok ? 0 : 1;
    }
    auto shortFile = std::vector<std::uint8_t>(10, 0);
    ok &= check(validate_xexe(shortFile, view) == XexeValidationError::TooSmall, "truncated header rejected");
    auto badMagic = file; badMagic[0] = 'Q';
    ok &= check(validate_xexe(badMagic, view) == XexeValidationError::BadMagic, "bad magic rejected");
    auto wrongAbi = file; put(wrongAbi, 14, 2, 2);
    ok &= check(validate_xexe(wrongAbi, view) == XexeValidationError::WrongAbi, "wrong ABI rejected");
    auto badEntry = file; put(badEntry, 16, 36, 8);
    ok &= check(validate_xexe(badEntry, view) == XexeValidationError::EntryOutsideCode, "entry at end rejected");
    auto badSize = file; put(badSize, 24, 8, 4);
    ok &= check(validate_xexe(badSize, view) == XexeValidationError::SizeMismatch, "oversized code rejected");
    auto badFlags = file; put(badFlags, 28, 1, 4);
    ok &= check(validate_xexe(badFlags, view) == XexeValidationError::ReservedFlagsSet, "reserved flags rejected");
    if (ok) std::cout << "Gravity .xexe validation tests: PASS\n";
    return ok ? 0 : 1;
}