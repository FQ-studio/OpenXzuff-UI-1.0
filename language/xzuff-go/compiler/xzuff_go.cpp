#include <array>
#include <charconv>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <string_view>
#include <system_error>
#include <utility>
#include <vector>

#include <gravity/abi.h>

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
                while (position_ < source_.size() &&
                       (isAlpha(source_[position_]) || isDigit(source_[position_]) || source_[position_] == '_')) {
                    value.push_back(source_[position_]);
                    advance();
                }
                tokens.push_back({value, startLine, startColumn});
                continue;
            }
            if (isDigit(c)) {
                std::string value;
                while (position_ < source_.size() && isDigit(source_[position_])) {
                    value.push_back(source_[position_]);
                    advance();
                }
                tokens.push_back({value, startLine, startColumn});
                continue;
            }
            if (std::string_view("(){}:;+-*/%").find(c) != std::string_view::npos) {
                tokens.push_back({std::string(1, c), startLine, startColumn});
                advance();
                continue;
            }
            throw std::runtime_error("unexpected character at line " + std::to_string(line_) +
                                     ", column " + std::to_string(column_));
        }
        tokens.push_back({"<end>", line_, column_});
        return tokens;
    }

private:
    static bool isAlpha(char c) { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'); }
    static bool isDigit(char c) { return c >= '0' && c <= '9'; }
    char peek(std::size_t offset) const {
        return position_ + offset < source_.size() ? source_[position_ + offset] : '\0';
    }
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
        const std::int64_t parsed = parseExpression();
        if (parsed < std::numeric_limits<std::int32_t>::min() ||
            parsed > std::numeric_limits<std::int32_t>::max()) {
            fail("constant expression result must fit signed i32 (-2147483648..2147483647)");
        }
        expect(";");
        expect("}");
        if (current().text != "<end>") {
            fail("only one function, main, is supported by this compiler milestone");
        }
        return static_cast<std::int32_t>(parsed);
    }

private:
    static bool isDigit(char c) { return c >= '0' && c <= '9'; }

    const Token& current() const { return tokens_.at(index_); }

    [[noreturn]] void fail(const std::string& message) const {
        throw std::runtime_error(message + " at line " + std::to_string(current().line) +
                                 ", column " + std::to_string(current().column));
    }

    void expect(const std::string& expected) {
        if (current().text != expected) {
            fail("expected '" + expected + "', found '" + current().text + "'");
        }
        ++index_;
    }

    bool match(std::string_view expected) {
        if (current().text != expected) return false;
        ++index_;
        return true;
    }

    std::int64_t checkedAdd(std::int64_t left, std::int64_t right) const {
        const auto minimum = std::numeric_limits<std::int64_t>::min();
        const auto maximum = std::numeric_limits<std::int64_t>::max();
        if ((right > 0 && left > maximum - right) ||
            (right < 0 && left < minimum - right)) {
            fail("signed 64-bit constant-expression overflow in addition");
        }
        return left + right;
    }

    std::int64_t checkedSubtract(std::int64_t left, std::int64_t right) const {
        const auto minimum = std::numeric_limits<std::int64_t>::min();
        const auto maximum = std::numeric_limits<std::int64_t>::max();
        if ((right > 0 && left < minimum + right) ||
            (right < 0 && left > maximum + right)) {
            fail("signed 64-bit constant-expression overflow in subtraction");
        }
        return left - right;
    }

    std::int64_t checkedMultiply(std::int64_t left, std::int64_t right) const {
        const auto minimum = std::numeric_limits<std::int64_t>::min();
        const auto maximum = std::numeric_limits<std::int64_t>::max();
        if (left == 0 || right == 0) return 0;
        if ((left == -1 && right == minimum) || (right == -1 && left == minimum)) {
            fail("signed 64-bit constant-expression overflow in multiplication");
        }
        if (left > 0) {
            if ((right > 0 && left > maximum / right) ||
                (right < 0 && right < minimum / left)) {
                fail("signed 64-bit constant-expression overflow in multiplication");
            }
        } else {
            if ((right > 0 && left < minimum / right) ||
                (right < 0 && right < maximum / left)) {
                fail("signed 64-bit constant-expression overflow in multiplication");
            }
        }
        return left * right;
    }

    std::int64_t checkedDivide(std::int64_t left, std::int64_t right) const {
        if (right == 0) fail("division by zero in constant expression");
        if (left == std::numeric_limits<std::int64_t>::min() && right == -1) {
            fail("signed 64-bit constant-expression overflow in division");
        }
        return left / right;
    }

    std::int64_t checkedModulo(std::int64_t left, std::int64_t right) const {
        if (right == 0) fail("modulo by zero in constant expression");
        if (left == std::numeric_limits<std::int64_t>::min() && right == -1) {
            fail("signed 64-bit constant-expression overflow in modulo");
        }
        return left % right;
    }

    std::int64_t parseExpression() { return parseAdditive(); }

    std::int64_t parseAdditive() {
        std::int64_t value = parseMultiplicative();
        while (current().text == "+" || current().text == "-") {
            const std::string operation = current().text;
            ++index_;
            const std::int64_t right = parseMultiplicative();
            value = operation == "+" ? checkedAdd(value, right) : checkedSubtract(value, right);
        }
        return value;
    }

    std::int64_t parseMultiplicative() {
        std::int64_t value = parseUnary();
        while (current().text == "*" || current().text == "/" || current().text == "%") {
            const std::string operation = current().text;
            ++index_;
            const std::int64_t right = parseUnary();
            if (operation == "*") value = checkedMultiply(value, right);
            else if (operation == "/") value = checkedDivide(value, right);
            else value = checkedModulo(value, right);
        }
        return value;
    }

    std::int64_t parseUnary() {
        if (match("+")) return parseUnary();
        if (match("-")) {
            const std::int64_t value = parseUnary();
            if (value == std::numeric_limits<std::int64_t>::min()) {
                fail("signed 64-bit constant-expression overflow in unary minus");
            }
            return -value;
        }
        return parsePrimary();
    }

    std::int64_t parsePrimary() {
        if (match("(")) {
            const std::int64_t value = parseExpression();
            expect(")");
            return value;
        }

        const Token token = current();
        if (token.text.empty() || !isDigit(token.text.front())) {
            fail("expected an integer literal or '(' in constant expression");
        }
        std::int64_t value = 0;
        const char* begin = token.text.data();
        const char* end = begin + token.text.size();
        const auto parsed = std::from_chars(begin, end, value, 10);
        if (parsed.ec == std::errc::result_out_of_range || parsed.ptr != end) {
            fail("integer literal exceeds signed 64-bit range");
        }
        if (parsed.ec != std::errc{}) fail("invalid integer literal");
        ++index_;
        return value;
    }

    std::vector<Token> tokens_;
    std::size_t index_ = 0;
};

#ifndef XZUFF_CHECKER_MODE
std::vector<std::uint8_t> generateRv64(std::int32_t returnValue) {
    // Gravity ABI v1: a0 carries the Exit status; a7 carries syscall ID 0.
    // LUI+ADDIW builds every signed i32 value and ADDIW sign-extends bit 31.
    const std::int64_t value = returnValue;
    const std::int64_t upper = (value + 0x800) >> 12;
    const std::int64_t lower = value - (upper * 4096);
    std::vector<std::uint8_t> code;
    const auto emit = [&code](std::uint32_t instruction) {
        code.push_back(static_cast<std::uint8_t>(instruction & 0xFFU));
        code.push_back(static_cast<std::uint8_t>((instruction >> 8U) & 0xFFU));
        code.push_back(static_cast<std::uint8_t>((instruction >> 16U) & 0xFFU));
        code.push_back(static_cast<std::uint8_t>((instruction >> 24U) & 0xFFU));
    };
    if (upper == 0) {
        emit((static_cast<std::uint32_t>(lower) & 0xFFFU) << 20U |
             (10U << 7U) | 0x13U);
    } else {
        const auto upper20 = static_cast<std::uint32_t>(upper) & 0xFFFFFU;
        emit((upper20 << 12U) | (10U << 7U) | 0x37U);
        if (lower != 0) {
            const auto lower12 = static_cast<std::uint32_t>(lower) & 0xFFFU;
            emit((lower12 << 20U) | (10U << 15U) | (10U << 7U) | 0x1BU);
        }
    }
    emit((static_cast<std::uint32_t>(GRAVITY_SYS_EXIT) << 20U) |
         (17U << 7U) | 0x13U);
    emit(0x00000073U);
    return code;
}

void writeLe(std::vector<std::uint8_t>& bytes, std::size_t offset,
             std::uint64_t value, std::size_t width) {
    if (offset > bytes.size() || width > bytes.size() - offset) {
        throw std::runtime_error("internal .xexe layout overflow");
    }
    for (std::size_t i = 0; i < width; ++i) {
        bytes[offset + i] = static_cast<std::uint8_t>((value >> (8U * i)) & 0xFFU);
    }
}

std::vector<std::uint8_t> makeGravityXexe(const std::vector<std::uint8_t>& code) {
    if (code.empty()) throw std::runtime_error("cannot emit an .xexe with an empty code payload");
    if (code.size() > std::numeric_limits<std::uint32_t>::max()) {
        throw std::runtime_error(".xexe code payload exceeds the v1 size field");
    }
    constexpr std::size_t headerSize = GRAVITY_XEXE_HEADER_SIZE;
    if (code.size() > std::numeric_limits<std::size_t>::max() - headerSize) {
        throw std::runtime_error(".xexe image size overflow");
    }
    std::vector<std::uint8_t> executable(headerSize + code.size(), 0);
    constexpr char magic[] = GRAVITY_XEXE_MAGIC;
    static_assert(sizeof(magic) == 8, "Gravity .xexe magic must remain exactly 8 bytes");
    for (std::size_t i = 0; i < sizeof(magic); ++i) {
        executable[i] = static_cast<std::uint8_t>(magic[i]);
    }
    writeLe(executable, GRAVITY_XEXE_OFFSET_FORMAT_VERSION, GRAVITY_XEXE_FORMAT_VERSION, 2);
    writeLe(executable, GRAVITY_XEXE_OFFSET_HEADER_SIZE, headerSize, 2);
    writeLe(executable, GRAVITY_XEXE_OFFSET_ARCHITECTURE, GRAVITY_XEXE_ARCH_RV64, 2);
    writeLe(executable, GRAVITY_XEXE_OFFSET_ABI_VERSION, GRAVITY_ABI_VERSION, 2);
    writeLe(executable, GRAVITY_XEXE_OFFSET_ENTRY, headerSize, 8);
    writeLe(executable, GRAVITY_XEXE_OFFSET_CODE_SIZE, code.size(), 4);
    writeLe(executable, GRAVITY_XEXE_OFFSET_FLAGS, GRAVITY_XEXE_RESERVED_FLAGS, 4);
    for (std::size_t i = 0; i < code.size(); ++i) executable[headerSize + i] = code[i];
    return executable;
}

int selfTest() {
    const auto zero = generateRv64(0);
    const std::vector<std::uint8_t> expectedZero{
        0x13, 0x05, 0x00, 0x00, 0x93, 0x08, 0x00, 0x00, 0x73, 0x00, 0x00, 0x00
    };
    if (zero != expectedZero) {
        std::cerr << "FAIL: RV64 instruction encoding for return 0\n";
        return 1;
    }

    const auto fortyTwo = generateRv64(42);
    const std::vector<std::uint8_t> expectedFortyTwo{
        0x13, 0x05, 0xA0, 0x02, 0x93, 0x08, 0x00, 0x00, 0x73, 0x00, 0x00, 0x00
    };
    if (fortyTwo != expectedFortyTwo) {
        std::cerr << "FAIL: RV64 instruction encoding for return 42\n";
        return 1;
    }

    const auto large = generateRv64(65536);
    const std::vector<std::uint8_t> expectedLarge{
        0x37, 0x05, 0x01, 0x00, 0x93, 0x08, 0x00, 0x00, 0x73, 0x00, 0x00, 0x00
    };
    if (large != expectedLarge) {
        std::cerr << "FAIL: RV64 instruction encoding for return 65536\n";
        return 1;
    }

    const auto negative = generateRv64(std::numeric_limits<std::int32_t>::min());
    const std::vector<std::uint8_t> expectedNegative{
        0x37, 0x05, 0x00, 0x80, 0x93, 0x08, 0x00, 0x00, 0x73, 0x00, 0x00, 0x00
    };
    if (negative != expectedNegative) {
        std::cerr << "FAIL: RV64 instruction encoding for INT32_MIN\n";
        return 1;
    }

    const auto maximum = generateRv64(std::numeric_limits<std::int32_t>::max());
    const std::vector<std::uint8_t> expectedMaximum{
        0x37, 0x05, 0x00, 0x80,
        0x1B, 0x05, 0xF5, 0xFF,
        0x93, 0x08, 0x00, 0x00,
        0x73, 0x00, 0x00, 0x00
    };
    if (maximum != expectedMaximum) {
        std::cerr << "FAIL: RV64 instruction encoding for INT32_MAX\n";
        return 1;
    }

    const auto parse = [](std::string source) {
        return Parser(Lexer(std::move(source)).scan()).parseMainReturn();
    };
    struct ExpressionCase {
        const char* source;
        std::int32_t expected;
    };
    const std::array<ExpressionCase, 6> expressions{{
        {"function main() returns i32 { return 2 + 3 * 4; }", 14},
        {"function main() returns i32 { return (2 + 3) * 4; }", 20},
        {"function main() returns i32 { return -(-7 + 2) * 3; }", 15},
        {"function main() returns i32 { return 18 / 3 / 2 + 17 % 5; }", 5},
        {"function main() returns i32 { return 1 - -2; }", 3},
        {"function main() returns i32 { return -2147483648; }", std::numeric_limits<std::int32_t>::min()}
    }};
    for (const auto& test : expressions) {
        if (parse(test.source) != test.expected) {
            std::cerr << "FAIL: constant-expression parser or precedence\n";
            return 1;
        }
    }

    const std::array<const char*, 4> invalidExpressions{{
        "function main() returns i32 { return 1 / (2 - 2); }",
        "function main() returns i32 { return 2147483647 + 1; }",
        "function main() returns i32 { return 9223372036854775807 + 1; }",
        "function main() returns i32 { return 2 + * 3; }"
    }};
    for (const char* source : invalidExpressions) {
        bool rejected = false;
        try {
            (void)parse(source);
        } catch (const std::exception&) {
            rejected = true;
        }
        if (!rejected) {
            std::cerr << "FAIL: invalid constant expression was accepted\n";
            return 1;
        }
    }

    std::cout << "Xzuff Go compiler self-test: PASS (RV64 codegen and constant expressions)\n";
    return 0;
}
#endif // XZUFF_CHECKER_MODE

} // namespace

#ifdef XZUFF_CHECKER_MODE
std::string jsonEscape(std::string_view value) {
    std::string escaped;
    for (unsigned char ch : value) {
        switch (ch) {
        case '"': escaped += "\\\""; break;
        case '\\': escaped += "\\\\"; break;
        case '\n': escaped += "\\n"; break;
        case '\r': escaped += "\\r"; break;
        case '\t': escaped += "\\t"; break;
        default:
            if (ch < 0x20U) {
                constexpr char hex[] = "0123456789abcdef";
                escaped += "\\u00";
                escaped.push_back(hex[(ch >> 4U) & 0xFU]);
                escaped.push_back(hex[ch & 0xFU]);
            } else {
                escaped.push_back(static_cast<char>(ch));
            }
        }
    }
    return escaped;
}
#endif // XZUFF_CHECKER_MODE

int main(int argc, char** argv) {
#ifdef XZUFF_CHECKER_MODE
    bool json = false;
    if (argc == 2 && std::string_view(argv[1]) == "--help") {
        std::cout << "Usage: xzuff-check [--json] <input.xgo>\n";
        return 0;
    }
    int sourceArg = 1;
    if (argc > 1 && std::string_view(argv[1]) == "--json") {
        json = true;
        sourceArg = 2;
    }
    if (argc != sourceArg + 1) {
        std::cerr << "Usage: xzuff-check [--json] <input.xgo>\n";
        return 2;
    }
    const std::string inputPath = argv[sourceArg];
#else
    if (argc == 2 && std::string_view(argv[1]) == "--self-test") return selfTest();
    if (argc == 3 && std::string_view(argv[1]) == "--check") {
        const std::string inputPath = argv[2];
        try {
            std::ifstream input(inputPath, std::ios::binary);
            if (!input) throw std::runtime_error("cannot open input source file");
            const std::string source((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
            Parser parser(Lexer(source).scan());
            (void)parser.parseMainReturn();
            std::cout << "Xzuff Go check: PASS (" << inputPath << ")\n";
            return 0;
        } catch (const std::exception& error) {
            std::cerr << "Xzuff Go check: FAIL (" << inputPath << "): " << error.what() << '\n';
            return 1;
        }
    }
    if (argc != 3) {
        std::cerr << "Usage: xzuff-go <input.xgo> <output.xexe>\n"
                  << "       xzuff-go --check <input.xgo>\n"
                  << "Supported subset: function main() returns i32 { return CONSTANT_EXPRESSION; }\n";
        return 2;
    }
    const std::string inputPath = argv[1];
#endif

    try {
        std::ifstream input(inputPath, std::ios::binary);
        if (!input) throw std::runtime_error("cannot open input source file");
        const std::string source((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
        Parser parser(Lexer(source).scan());
        const auto returnValue = parser.parseMainReturn();
#ifdef XZUFF_CHECKER_MODE
        if (json) {
            std::cout << "{\"ok\":true,\"file\":\"" << jsonEscape(inputPath)
                      << "\",\"return_value\":" << returnValue << ",\"diagnostics\":[]}\n";
        } else {
            std::cout << "Xzuff Go check: PASS (" << inputPath << ", return " << returnValue << ")\n";
        }
        return 0;
#else
        const auto code = generateRv64(returnValue);
        const auto executable = makeGravityXexe(code);
        std::ofstream output(argv[2], std::ios::binary);
        if (!output) throw std::runtime_error("cannot create output file");
        output.write(reinterpret_cast<const char*>(executable.data()),
                     static_cast<std::streamsize>(executable.size()));
        if (!output) throw std::runtime_error("failed while writing output file");
        std::cout << "Generated Gravity Kernel .xexe executable (" << executable.size()
                  << " bytes) with RV64 machine code. Entry function exits with status "
                  << returnValue << ".\n";
        std::cout << "Target ABI is Gravity Kernel RV64 ABI v1 (Exit syscall ID "
                  << GRAVITY_SYS_EXIT << ").\n";
        return 0;
#endif
    } catch (const std::exception& error) {
#ifdef XZUFF_CHECKER_MODE
        if (json) {
            std::cout << "{\"ok\":false,\"file\":\"" << jsonEscape(inputPath)
                      << "\",\"diagnostics\":[{\"message\":\"" << jsonEscape(error.what()) << "\"}]}\n";
        } else {
            std::cerr << "Xzuff Go check: FAIL (" << inputPath << "): " << error.what() << '\n';
        }
#else
        std::cerr << "Xzuff Go compiler error: " << error.what() << '\n';
#endif
        return 1;
    }
}
