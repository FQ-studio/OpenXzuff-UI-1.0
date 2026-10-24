#include <array>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <string_view>
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

std::array<std::uint8_t, 8> generateRv64(std::int32_t returnValue) {
    const auto immediate = static_cast<std::uint32_t>(returnValue) & 0xFFFU;
    const std::uint32_t addiA0 = (immediate << 20U) | (10U << 7U) | 0x13U; // addi a0, zero, imm
    const std::uint32_t ret = 0x00008067U; // jalr zero, ra, 0
    return {
        static_cast<std::uint8_t>(addiA0 & 0xFFU),
        static_cast<std::uint8_t>((addiA0 >> 8U) & 0xFFU),
        static_cast<std::uint8_t>((addiA0 >> 16U) & 0xFFU),
        static_cast<std::uint8_t>((addiA0 >> 24U) & 0xFFU),
        static_cast<std::uint8_t>(ret & 0xFFU),
        static_cast<std::uint8_t>((ret >> 8U) & 0xFFU),
        static_cast<std::uint8_t>((ret >> 16U) & 0xFFU),
        static_cast<std::uint8_t>((ret >> 24U) & 0xFFU)
    };
}

int selfTest() {
    const auto zero = generateRv64(0);
    const std::array<std::uint8_t, 8> expectedZero{0x13, 0x05, 0x00, 0x00, 0x67, 0x80, 0x00, 0x00};
    if (zero != expectedZero) {
        std::cerr << "FAIL: RV64 instruction encoding for return 0\n";
        return 1;
    }
    const auto fortyTwo = generateRv64(42);
    const std::array<std::uint8_t, 8> expectedFortyTwo{0x13, 0x05, 0xA0, 0x02, 0x67, 0x80, 0x00, 0x00};
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
            std::cerr << "Usage: xzuff-go <input.xgo> <output.rv64.bin>\n"
                      << "Supported subset: function main() returns i32 { return INTEGER; }\n";
            return 2;
        }
        std::ifstream input(argv[1], std::ios::binary);
        if (!input) throw std::runtime_error("cannot open input source file");
        const std::string source((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
        Parser parser(Lexer(source).scan());
        const auto returnValue = parser.parseMainReturn();
        const auto code = generateRv64(returnValue);
        std::ofstream output(argv[2], std::ios::binary);
        if (!output) throw std::runtime_error("cannot create output file");
        output.write(reinterpret_cast<const char*>(code.data()), static_cast<std::streamsize>(code.size()));
        if (!output) throw std::runtime_error("failed while writing output file");
        std::cout << "Generated " << code.size() << " bytes of RV64 machine code. Entry function returns " << returnValue << ".\n";
        std::cout << "Output is a raw code fragment, not an ELF executable and not yet loadable by Gravity Kernel.\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << "Xzuff Go compiler error: " << error.what() << '\n';
        return 1;
    }
}
