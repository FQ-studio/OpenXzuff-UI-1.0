#include <Luau/Compiler.h>
#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>

#include <array>
#include <chrono>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <string_view>
#include <vector>

namespace fs = std::filesystem;
namespace {
constexpr int kInstructionLimit = 2'000'000;
constexpr int kHookInterval = 1'000;
constexpr std::string_view kMagic = "XZLUAP1\n";
constexpr std::uint32_t kMaxManifestBytes = 64 * 1024;
constexpr std::uint64_t kMaxSourceBytes = 16 * 1024 * 1024;

struct ExecutionBudget { int remaining = kInstructionLimit; };

void instructionInterrupt(lua_State* state, int) {
    auto* callbacks = lua_callbacks(state);
    auto* budget = callbacks ? static_cast<ExecutionBudget*>(callbacks->userdata) : nullptr;
    if (!budget) luaL_error(state, "Xzuff Runtime: execution budget unavailable");
    budget->remaining -= kHookInterval;
    if (budget->remaining <= 0) luaL_error(state, "Xzuff Runtime: script execution budget exceeded");
}

bool readFile(const fs::path& path, std::string& output) {
    std::ifstream file(path, std::ios::binary);
    if (!file) return false;
    output.assign(std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>());
    return file.good() || file.eof();
}
bool writeU32(std::ostream& out, std::uint32_t value) {
    for (int i = 0; i < 4; ++i) out.put(static_cast<char>((value >> (i * 8)) & 0xff));
    return static_cast<bool>(out);
}
bool writeU64(std::ostream& out, std::uint64_t value) {
    for (int i = 0; i < 8; ++i) out.put(static_cast<char>((value >> (i * 8)) & 0xff));
    return static_cast<bool>(out);
}
bool readU32(std::istream& in, std::uint32_t& value) {
    value = 0;
    for (int i = 0; i < 4; ++i) { int c = in.get(); if (c == EOF) return false; value |= std::uint32_t(static_cast<unsigned char>(c)) << (i * 8); }
    return true;
}
bool readU64(std::istream& in, std::uint64_t& value) {
    value = 0;
    for (int i = 0; i < 8; ++i) { int c = in.get(); if (c == EOF) return false; value |= std::uint64_t(static_cast<unsigned char>(c)) << (i * 8); }
    return true;
}

struct Package { std::string manifest; std::string source; };
bool readPackage(const fs::path& path, Package& pkg, std::string& error) {
    std::ifstream in(path, std::ios::binary);
    if (!in) { error = "cannot open package"; return false; }
    std::array<char, 8> magic{};
    in.read(magic.data(), static_cast<std::streamsize>(magic.size()));
    if (!in || std::string_view(magic.data(), magic.size()) != kMagic) { error = "invalid .luap magic/version"; return false; }
    std::uint32_t manifestSize = 0;
    std::uint64_t sourceSize = 0;
    if (!readU32(in, manifestSize) || !readU64(in, sourceSize)) { error = "truncated package header"; return false; }
    if (manifestSize == 0 || manifestSize > kMaxManifestBytes || sourceSize == 0 || sourceSize > kMaxSourceBytes) {
        error = "package sizes exceed limits"; return false;
    }
    pkg.manifest.resize(manifestSize);
    pkg.source.resize(static_cast<std::size_t>(sourceSize));
    in.read(pkg.manifest.data(), static_cast<std::streamsize>(manifestSize));
    in.read(pkg.source.data(), static_cast<std::streamsize>(sourceSize));
    if (!in) { error = "package payload is truncated"; return false; }
    if (in.peek() != EOF) { error = "unexpected trailing data"; return false; }
    if (pkg.manifest.find("\"entry\":\"main.luau\"") == std::string::npos) {
        error = "manifest must declare entry main.luau"; return false;
    }
    return true;
}
bool packScript(const fs::path& input, const fs::path& output, const std::string& name, const std::string& version) {
    std::string source;
    if (!readFile(input, source) || source.empty() || source.size() > kMaxSourceBytes) {
        std::cerr << "Xzuff Runtime: cannot read source or source exceeds 16 MiB.\n"; return false;
    }
    if (name.empty() || name.find_first_of("\r\n\"") != std::string::npos ||
        version.empty() || version.find_first_of("\r\n\"") != std::string::npos) {
        std::cerr << "Xzuff Runtime: package name/version contains invalid characters.\n"; return false;
    }
    const std::string manifest = "{\"format\":\"xzuff-luap\",\"version\":1,\"name\":\"" + name +
        "\",\"packageVersion\":\"" + version + "\",\"entry\":\"main.luau\"}";
    std::ofstream out(output, std::ios::binary | std::ios::trunc);
    if (!out) { std::cerr << "Xzuff Runtime: cannot create output package.\n"; return false; }
    out.write(kMagic.data(), static_cast<std::streamsize>(kMagic.size()));
    writeU32(out, static_cast<std::uint32_t>(manifest.size()));
    writeU64(out, static_cast<std::uint64_t>(source.size()));
    out.write(manifest.data(), static_cast<std::streamsize>(manifest.size()));
    out.write(source.data(), static_cast<std::streamsize>(source.size()));
    if (!out) { std::cerr << "Xzuff Runtime: failed writing package.\n"; return false; }
    std::cout << "Created .luap package: " << output.string() << "\n";
    return true;
}

int runSource(const std::string& source, const std::string& chunkName) {
    lua_State* state = luaL_newstate();
    if (!state) { std::cerr << "Xzuff Runtime: could not create Luau VM.\n"; return 1; }

    // Register the VM safepoint callback before creating globals/libraries.
    // The budget is approximate: Luau calls this at loop/call/return/GC safepoints.
    ExecutionBudget budget;
    lua_Callbacks* callbacks = lua_callbacks(state);
    callbacks->userdata = &budget;
    callbacks->interrupt = instructionInterrupt;

    // v0.2 uses Luau's built-in libraries and sandboxing; this is not OS isolation.
    luaL_openlibs(state);
    luaL_sandbox(state);

    Luau::CompileOptions options;
    options.optimizationLevel = 1;
    options.debugLevel = 1;
    const std::string bytecode = Luau::compile(source, options);
    const auto started = std::chrono::steady_clock::now();
    const int loadStatus = luau_load(state, chunkName.c_str(), bytecode.data(), bytecode.size(), 0);
    if (loadStatus != 0) {
        const char* message = lua_tostring(state, -1);
        std::cerr << "Xzuff Runtime: compile/load error: " << (message ? message : "unknown error") << '\n';
        lua_close(state); return 1;
    }
    const int callStatus = lua_pcall(state, 0, LUA_MULTRET, 0);
    const auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
        std::chrono::steady_clock::now() - started).count();
    if (callStatus != 0) {
        const char* message = lua_tostring(state, -1);
        std::cerr << "Xzuff Runtime: script error: " << (message ? message : "unknown error") << '\n';
        lua_close(state); return 1;
    }
    std::cout << "Xzuff Runtime: script finished (" << elapsed << " ms).\n";
    lua_close(state); return 0;
}
void usage(const char* program) {
    std::cerr << "Xzuff Luau Runtime 0.2.0\n"
              << "Run:  " << program << " <script.luau|script.lua|game.luap>\n"
              << "Pack: " << program << " --pack <source.luau> <output.luap> [name] [version]\n";
}
} // namespace

int main(int argc, char** argv) {
    if (argc >= 2 && std::string_view(argv[1]) == "--pack") {
        if (argc < 4 || argc > 6) { usage(argv[0]); return 2; }
        const std::string name = argc >= 5 ? argv[4] : fs::path(argv[3]).stem().string();
        const std::string version = argc >= 6 ? argv[5] : "1.0.0";
        return packScript(argv[2], argv[3], name, version) ? 0 : 1;
    }
    if (argc != 2) { usage(argv[0]); return 2; }
    const fs::path path(argv[1]);
    const std::string extension = path.extension().string();
    std::string source;
    if (extension == ".luap") {
        Package pkg; std::string error;
        if (!readPackage(path, pkg, error)) {
            std::cerr << "Xzuff Runtime: invalid .luap package: " << error << '\n'; return 1;
        }
        std::cout << "Loading game package: " << path.filename().string() << '\n';
        return runSource(pkg.source, path.string() + ":main.luau");
    }
    if (extension != ".luau" && extension != ".lua") {
        std::cerr << "Xzuff Runtime: expected .luau, .lua or .luap.\n"; return 2;
    }
    if (!readFile(path, source)) {
        std::cerr << "Xzuff Runtime: cannot read file: " << path.string() << '\n'; return 1;
    }
    return runSource(source, path.string());
}
