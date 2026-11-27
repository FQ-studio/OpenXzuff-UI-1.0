#include "openxzuff/kernel/xexe_loader.hpp"
#include <array>
#include <charconv>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <system_error>
#include <vector>

namespace {
void put(std::vector<std::uint8_t>& bytes, std::size_t offset,
         std::uint64_t value, std::size_t width) {
    for (std::size_t i = 0; i < width; ++i) {
        bytes[offset + i] = static_cast<std::uint8_t>((value >> (8U * i)) & 0xFFU);
    }
}

std::vector<std::uint8_t> validFile() {
    std::vector<std::uint8_t> file(GRAVITY_XEXE_HEADER_SIZE + 4, 0);
    constexpr char magic[] = GRAVITY_XEXE_MAGIC;
    for (std::size_t i = 0; i < sizeof(magic); ++i) {
        file[i] = static_cast<std::uint8_t>(magic[i]);
    }
    put(file, GRAVITY_XEXE_OFFSET_FORMAT_VERSION, GRAVITY_XEXE_FORMAT_VERSION, 2);
    put(file, GRAVITY_XEXE_OFFSET_HEADER_SIZE, GRAVITY_XEXE_HEADER_SIZE, 2);
    put(file, GRAVITY_XEXE_OFFSET_ARCHITECTURE, GRAVITY_XEXE_ARCH_RV64, 2);
    put(file, GRAVITY_XEXE_OFFSET_ABI_VERSION, GRAVITY_ABI_VERSION, 2);
    put(file, GRAVITY_XEXE_OFFSET_ENTRY, GRAVITY_XEXE_HEADER_SIZE, 8);
    put(file, GRAVITY_XEXE_OFFSET_CODE_SIZE, 4, 4);
    file[GRAVITY_XEXE_HEADER_SIZE] = 0x13;
    file[GRAVITY_XEXE_HEADER_SIZE + 1] = 0x05;
    file[GRAVITY_XEXE_HEADER_SIZE + 2] = 0x00;
    file[GRAVITY_XEXE_HEADER_SIZE + 3] = 0x00;
    return file;
}

std::vector<std::uint8_t> expectedExitCode(std::int32_t status) {
    const std::int64_t value = status;
    const std::int64_t upper = (value + 0x800) >> 12;
    const std::int64_t lower = value - upper * 4096;
    std::vector<std::uint8_t> code;
    const auto emit = [&code](std::uint32_t instruction) {
        code.push_back(static_cast<std::uint8_t>(instruction & 0xFFU));
        code.push_back(static_cast<std::uint8_t>((instruction >> 8U) & 0xFFU));
        code.push_back(static_cast<std::uint8_t>((instruction >> 16U) & 0xFFU));
        code.push_back(static_cast<std::uint8_t>((instruction >> 24U) & 0xFFU));
    };
    if (upper == 0) {
        emit((static_cast<std::uint32_t>(lower) & 0xFFFU) << 20U | (10U << 7U) | 0x13U);
    } else {
        const auto upper20 = static_cast<std::uint32_t>(upper) & 0xFFFFFU;
        emit((upper20 << 12U) | (10U << 7U) | 0x37U);
        if (lower != 0) {
            const auto lower12 = static_cast<std::uint32_t>(lower) & 0xFFFU;
            emit((lower12 << 20U) | (10U << 15U) | (10U << 7U) | 0x1BU);
        }
    }
    emit((static_cast<std::uint32_t>(GRAVITY_SYS_EXIT) << 20U) | (17U << 7U) | 0x13U);
    emit(0x00000073U);
    return code;
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
    if (argc == 2 || argc == 3) {
        std::ifstream input(argv[1], std::ios::binary);
        if (!input) {
            std::cerr << "FAIL: cannot open .xexe input file\n";
            return 1;
        }
        file.assign(std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>());
    } else if (argc != 1) {
        std::cerr << "Usage: xexe-loader-test [program.xexe [expected-exit-status]]\n";
        return 2;
    }

    std::int32_t expectedStatus = 42;
    if (argc == 3) {
        const std::string_view text(argv[2]);
        const auto parsed = std::from_chars(text.data(), text.data() + text.size(), expectedStatus, 10);
        if (parsed.ec != std::errc{} || parsed.ptr != text.data() + text.size()) {
            std::cerr << "FAIL: expected exit status must be a signed i32 integer\n";
            return 2;
        }
    }

    XexeView view{};
    ok &= check(validate_xexe(file, view) == XexeValidationError::None, "valid image accepted");
    ok &= check(!view.code.empty() && view.entry_offset == GRAVITY_XEXE_HEADER_SIZE,
                "code view and entry offset");
    if (argc >= 2) {
        const auto expected = expectedExitCode(expectedStatus);
        ok &= check(view.code.size() == expected.size(), "compiler-produced RV64 code size");
        if (view.code.size() == expected.size()) {
            for (std::size_t i = 0; i < expected.size(); ++i) {
                ok &= check(view.code[i] == expected[i], "compiler-produced RV64 instruction bytes");
            }
        }
        if (ok) {
            std::cout << "Compiler-produced .xexe validation and RV64 exit-code test: PASS ("
                      << expectedStatus << ")\n";
        }
        return ok ? 0 : 1;
    }

    auto shortFile = std::vector<std::uint8_t>(10, 0);
    ok &= check(validate_xexe(shortFile, view) == XexeValidationError::TooSmall,
                "truncated header rejected");
    auto badMagic = file;
    badMagic[0] = 'Q';
    ok &= check(validate_xexe(badMagic, view) == XexeValidationError::BadMagic, "bad magic rejected");
    auto wrongAbi = file;
    put(wrongAbi, GRAVITY_XEXE_OFFSET_ABI_VERSION, 2, 2);
    ok &= check(validate_xexe(wrongAbi, view) == XexeValidationError::WrongAbi, "wrong ABI rejected");
    auto badEntry = file;
    put(badEntry, GRAVITY_XEXE_OFFSET_ENTRY, GRAVITY_XEXE_HEADER_SIZE + 4, 8);
    ok &= check(validate_xexe(badEntry, view) == XexeValidationError::EntryOutsideCode,
                "entry at end rejected");
    auto badSize = file;
    put(badSize, GRAVITY_XEXE_OFFSET_CODE_SIZE, 8, 4);
    ok &= check(validate_xexe(badSize, view) == XexeValidationError::SizeMismatch,
                "oversized code rejected");
    auto badFlags = file;
    put(badFlags, GRAVITY_XEXE_OFFSET_FLAGS, 1, 4);
    ok &= check(validate_xexe(badFlags, view) == XexeValidationError::ReservedFlagsSet,
                "reserved flags rejected");
    if (ok) std::cout << "Gravity .xexe validation tests: PASS\n";
    return ok ? 0 : 1;
}
