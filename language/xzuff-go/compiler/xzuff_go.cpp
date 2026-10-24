#include <array>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace {

struct Token {
    std::string text;
    std::size_t line;
    std::size_t column;
};

class Lexer {
public:
    explicit Lexer(std::string source) : source_(std::move(source)) {}

    std::vector<Token> scan() {
        std::vector<Token> tokens;
        while (position_ < source_.size()) {
            const char c = source_[position_];
            if (c == ' ' || c == '\t' || c == '\r') { advance(); continue; }
            if (c == '\n') { advance(); ++line_; column_ = 1; continue; }
            if (c == '/' && peek(1) == '/') {
                while (position_ < source_.size() && source_[position_] != '\n') advance();
                continue;
            }
            const std::size_t startLine = line_;
            const std::size_t startColumn = column_;
            if (isAlpha(c) || c == '_') {
                std::string value;
                while (position_ < source_.size() && (isAlpha(source_[position_]) || isDigit(source_[position_]) || source_[position_] == '_')) {
                    value.push_back(source_[position_]);
                    advance();
                }
                tokens.push_back({value, startLine, startColumn});
                continue;
            }
            if (isDigit(c) || (c == '-' && isDigit(peek(1)))) {
                std::string value;
                if (c == '-') { value.push_back(c); advance(); }
                while (position_ < source_.size() && isDigit(source_[position_])) {
                    value.push_back(source_[position_]);
                    advance();
                }
                tokens.push_back({value, startLine, startColumn});
                continue;
            }
            if (std::string_view("(){}:;").find(c) != std::string_view::npos) {
                tokens.push_back({std::string(1, c), startLine, startColumn});
                advance();
                continue;
            }
            throw std::runtime_error("unexpected character at line " + std::to_string(line_) + ", column " + std::to_string(column_));
        }
        tokens.push_back({"<end>", line_, column_});
        return tokens;
    }

private:
    static bool isAlpha(char c) { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'); }
    static bool isDigit(char c) { return c >= '0' && c <= '9'; }
    char peek(std::size_t offset) const { return position_ + offset < source_.size() ? source_[position_ + offset] : '\0'; }
    void advance() { ++position_; ++column_; }

    std::string source_;
    std::size_t position_ = 0;
    std::size_t line_ = 1;
    std::size_t column_ = 1;
};

class Parser {
public:
    explicit Parser(std::vector<Token> tokens) : tokens_(std::move(tokens)) {}

    std::int32_t parseMainReturn() {
        expect("function");
        expect("main");
        expect("(");
        expect(")");
        expect("returns");
        expect("i32");
        expect("{");
        expect("return");
        const Token value = current();
        std::int64_t parsed = 0;
        try {
            std::size_t consumed = 0;
            parsed = std::stoll(value.text, &consumed, 10);
            if (consumed != value.text.size()) fail("expected an integer return value");
        } catch (const std::exception&) {
            fail("expected an integer return value");
        }
        if (parsed < -2048 || parsed > 2047) fail("return constant must fit the RV64 addi signed 12-bit immediate (-2048..2047)");
        ++index_;
        expect(";");
        expect("}");
        if (current().text != "<end>") fail("only one function, main, is supported by this compiler milestone");
        return static_cast<std::int32_t>(parsed);
    }

private:
    const Token& current() const { return tokens_.at(index_); }
    [[noreturn]] void fail(const std::string& message) const {
        throw std::runtime_error(message + " at line " + std::to_string(current().line) + ", column " + std::to_string(current().column));
    }
    void expect(const std::string& expected) {
        if (current().text != expected) fail("expected '" + expected + "', found '" + current().text + "'");
        ++index_;
    }
    std::vector<Token> tokens_;
    std::size_t index_ = 0;
};

std::array<std::uint8_t, 12> generateRv64(std::int32_t returnValue) {
    // Gravity Kernel ABI v1: a0 carries the exit status; a7 = SyscallId::Exit (0).
    // The kernel ABI defines ecall as the syscall trap. Explicitly set a7 so entry register state is irrelevant.
    const auto immediate = static_cast<std::uint32_t>(returnValue) & 0xFFFU;
    const std::uint32_t addiA0 = (immediate << 20U) | (10U << 7U) | 0x13U; // addi a0, zero, imm
    const std::uint32_t addiA7 = (17U << 7U) | 0x13U; // addi a7, zero, 0 (Gravity Exit = 0)\n    const std::uint32_t ecall = 0x00000073U;
    return {
        static_cast<std::uint8_t>(addiA0 & 0xFFU),
        static_cast<std::uint8_t>((addiA0 >> 8U) & 0xFFU),
        static_cast<std::uint8_t>((addiA0 >> 16U) & 0xFFU),
        static_cast<std::uint8_t>((addiA0 >> 24U) & 0xFFU),
        static_cast<std::uint8_t>(ecall & 0xFFU),
        static_cast<std::uint8_t>((ecall >> 8U) & 0xFFU),
        static_cast<std::uint8_t>((ecall >> 16U) & 0xFFU),
        static_cast<std::uint8_t>((ecall >> 24U) & 0xFFU)
    };
}

void writeLe(std::vector<std::uint8_t>& bytes, std::size_t offset, std::uint64_t value, std::size_t width) {
    if (offset + width > bytes.size()) throw std::runtime_error("internal .xexe layout overflow");
    for (std::size_t i = 0; i < width; ++i) {
        bytes[offset + i] = static_cast<std::uint8_t>((value >> (8U * i)) & 0xFFU);
    }
}

std::vector<std::uint8_t> makeGravityXexe(const std::array<std::uint8_t, 12>& code) {
    // OpenXzuff .xexe v1 header: magic[8], version:u16, header_size:u16,
    // architecture:u16 (1 = RV64), ABI version:u16, entry_offset:u64,
    // code_size:u32, flags:u32. All integer fields are little-endian.
    constexpr std::size_t headerSize = 32;
    std::vector<std::uint8_t> executable(headerSize + code.size(), 0);
    const std::array<std::uint8_t, 8> magic{'X','Z','X','E','X','E','1',0};
    for (std::size_t i = 0; i < magic.size(); ++i) executable[i] = magic[i];
    writeLe(executable, 8, 1, 2); // format version
    writeLe(executable, 10, headerSize, 2);
    writeLe(executable, 12, 1, 2); // RV64
    writeLe(executable, 14, 1, 2); // Gravity Kernel ABI v1
    writeLe(executable, 16, headerSize, 8); // entry offset from start of file
    writeLe(executable, 24, code.size(), 4);
    writeLe(executable, 28, 0, 4); // flags reserved; must be zero in v1
    for (std::size_t i = 0; i < code.size(); ++i) executable[headerSize + i] = code[i];
    return executable;
}

int selfTest() {
    const auto zero = generateRv64(0);
    const std::array<std::uint8_t, 12> expectedZero{0x13, 0x05, 0x00, 0x00, 0x93, 0x08, 0x00, 0x00, 0x73, 0x00, 0x00, 0x00};
    if (zero != expectedZero) {
        std::cerr << "FAIL: RV64 instruction encoding for return 0\n";
        return 1;
    }
    const auto fortyTwo = generateRv64(42);
    const std::array<std::uint8_t, 12> expectedFortyTwo{0x13, 0x05, 0xA0, 0x02, 0x93, 0x08, 0x00, 0x00, 0x73, 0x00, 0x00, 0x00};
    if (fortyTwo != expectedFortyTwo) {
        std::cerr << "FAIL: RV64 instruction encoding for return 42\n";
        return 1;
    }
    std::cout << "Xzuff Go compiler unit self-test: PASS\n";
    return 0;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc == 2 && std::string_view(argv[1]) == "--self-test") return selfTest();
        if (argc != 3) {
            std::cerr << "Usage: xzuff-go <input.xgo> <output.xexe>\n"
                      << "Supported subset: function main() returns i32 { return INTEGER; }\n";
            return 2;
        }
        std::ifstream input(argv[1], std::ios::binary);
        if (!input) throw std::runtime_error("cannot open input source file");
        const std::string source((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
        Parser parser(Lexer(source).scan());
        const auto returnValue = parser.parseMainReturn();
        const auto code = generateRv64(returnValue);
        const auto executable = makeGravityXexe(code);
        std::ofstream output(argv[2], std::ios::binary);
        if (!output) throw std::runtime_error("cannot create output file");
        output.write(reinterpret_cast<const char*>(executable.data()), static_cast<std::streamsize>(executable.size()));
        if (!output) throw std::runtime_error("failed while writing output file");
        std::cout << "Generated Gravity Kernel .xexe executable (" << executable.size() << " bytes) with RV64 machine code. Entry function exits with status " << returnValue << ".\n";
        std::cout << "Target ABI is Gravity Kernel RV64 ABI v1 (Exit syscall ID 0).\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << "Xzuff Go compiler error: " << error.what() << '\n';
        return 1;
    }
}
