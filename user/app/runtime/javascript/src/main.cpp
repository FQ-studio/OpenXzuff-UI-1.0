#include <quickjs.h>

#include <charconv>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <limits>
#include <string>
#include <string_view>

namespace {
using Clock = std::chrono::steady_clock;
constexpr std::size_t kDefaultMemoryLimit = 16U * 1024U * 1024U;
constexpr std::size_t kMaxSourceBytes = 4U * 1024U * 1024U;
constexpr std::size_t kMaxStackBytes = 256U * 1024U;

struct ExecutionBudget {
    Clock::time_point deadline;
};

int interruptHandler(JSRuntime*, void* opaque) {
    const auto* budget = static_cast<const ExecutionBudget*>(opaque);
    return budget == nullptr || Clock::now() >= budget->deadline;
}

JSValue jsPrint(JSContext* context, JSValueConst, int argc, JSValueConst* argv) {
    for (int i = 0; i < argc; ++i) {
        if (i != 0) std::cout << ' ';
        std::size_t length = 0;
        const char* value = JS_ToCStringLen(context, &length, argv[i]);
        if (value == nullptr) return JS_EXCEPTION;
        std::cout.write(value, static_cast<std::streamsize>(length));
        JS_FreeCString(context, value);
    }
    std::cout << '\n';
    return JS_UNDEFINED;
}

bool parsePositive(std::string_view text, std::uint64_t& value) {
    if (text.empty()) return false;
    const auto result = std::from_chars(text.data(), text.data() + text.size(), value);
    return result.ec == std::errc{} && result.ptr == text.data() + text.size() && value > 0;
}

bool readSource(const std::string& path, std::string& source) {
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file) return false;
    const auto end = file.tellg();
    if (end <= 0 || static_cast<std::uint64_t>(end) > kMaxSourceBytes) return false;
    source.resize(static_cast<std::size_t>(end));
    file.seekg(0, std::ios::beg);
    file.read(source.data(), static_cast<std::streamsize>(source.size()));
    return static_cast<bool>(file);
}

void printException(JSContext* context) {
    JSValue exception = JS_GetException(context);
    const char* message = JS_ToCString(context, exception);
    std::cerr << "Xzuff JS: " << (message != nullptr ? message : "JavaScript execution failed") << '\n';
    if (message != nullptr) JS_FreeCString(context, message);
    JS_FreeValue(context, exception);
}

void usage(const char* program) {
    std::cerr << "Xzuff JavaScript Runtime 0.1.0 (host prototype)\n"
              << "Usage: " << program << " [--timeout-ms N] [--memory-mib N] <script.js>\n";
}
} // namespace

int main(int argc, char** argv) {
    std::uint64_t timeoutMs = 1000;
    std::uint64_t memoryMiB = kDefaultMemoryLimit / (1024U * 1024U);
    std::string scriptPath;

    for (int i = 1; i < argc; ++i) {
        const std::string_view arg(argv[i]);
        if ((arg == "--timeout-ms" || arg == "--memory-mib") && i + 1 < argc) {
            std::uint64_t parsed = 0;
            if (!parsePositive(argv[++i], parsed)) {
                usage(argv[0]);
                return 2;
            }
            if (arg == "--timeout-ms") timeoutMs = parsed;
            else memoryMiB = parsed;
        } else if (!arg.empty() && arg.front() == '-') {
            usage(argv[0]);
            return 2;
        } else if (scriptPath.empty()) {
            scriptPath = std::string(arg);
        } else {
            usage(argv[0]);
            return 2;
        }
    }

    if (scriptPath.empty() || timeoutMs > 60000 || memoryMiB > 1024) {
        usage(argv[0]);
        return 2;
    }

    std::string source;
    if (!readSource(scriptPath, source)) {
        std::cerr << "Xzuff JS: cannot read script or script exceeds 4 MiB: " << scriptPath << '\n';
        return 1;
    }

    JSRuntime* runtime = JS_NewRuntime();
    if (runtime == nullptr) {
        std::cerr << "Xzuff JS: cannot create JavaScript runtime.\n";
        return 1;
    }

    JS_SetMemoryLimit(runtime, static_cast<std::size_t>(memoryMiB) * 1024U * 1024U);
    JS_SetMaxStackSize(runtime, kMaxStackBytes);
    ExecutionBudget budget{Clock::now() + std::chrono::milliseconds(timeoutMs)};
    JS_SetInterruptHandler(runtime, interruptHandler, &budget);

    JSContext* context = JS_NewContext(runtime);
    if (context == nullptr) {
        std::cerr << "Xzuff JS: cannot create JavaScript context.\n";
        JS_FreeRuntime(runtime);
        return 1;
    }

    JSValue global = JS_GetGlobalObject(context);
    JS_SetPropertyStr(context, global, "print", JS_NewCFunction(context, jsPrint, "print", 1));
    JS_FreeValue(context, global);

    const JSValue result = JS_Eval(
        context, source.data(), source.size(), scriptPath.c_str(), JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(result)) {
        printException(context);
        JS_FreeContext(context);
        JS_FreeRuntime(runtime);
        return 1;
    }

    JS_FreeValue(context, result);
    JS_FreeContext(context);
    JS_FreeRuntime(runtime);
    return 0;
}
