#pragma once
#include <cstdint>
#include <string>
#include <string_view>
namespace gravity::ai {
enum class ArithmeticOperation { Add, Subtract, Multiply, Divide };
struct ArithmeticResult { bool ok{}; std::int64_t value{}; std::string error; };
class ArithmeticEngine {
public:
 ArithmeticResult evaluate(std::string_view expression) const;
};
class SentenceEngine {
public:
 std::string compose(std::string_view question, std::int64_t answer) const;
};
struct GailResult { bool ok{}; std::int64_t answer{}; std::string sentence; std::string error; };
class GailRuntime {
public:
 // Loads supported GAIL arithmetic declarations and runs their mechanisms.
 bool load(std::string_view source, std::string* error = nullptr);
 GailResult ask(std::string_view question) const;
 bool ready() const noexcept { return loaded_; }
private:
 bool loaded_{false};
 bool add_{false}, subtract_{false}, multiply_{false}, divide_{false};
 ArithmeticEngine arithmetic_;
 SentenceEngine sentence_;
};
} // namespace gravity::ai
