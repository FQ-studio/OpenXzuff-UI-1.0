extern "C" {
#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>
}

#include <cstddef>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <string_view>

namespace {
constexpr std::size_t kMemoryLimit = 16u * 1024u * 1024u;
constexpr std::size_t kSourceLimit = 4u * 1024u * 1024u;
constexpr int kInstructionLimit = 2'000'000;
constexpr int kHookInterval = 1'000;

struct MemoryState { std::size_t used = 0; };

void* limitedAllocator(void* opaque, void* pointer, std::size_t oldSize, std::size_t newSize) {
    auto* memory = static_cast<MemoryState*>(opaque);
    const std::size_t actualOld = pointer ? oldSize : 0;
    if (newSize == 0) {
        if (pointer) {
            std::free(pointer);
            memory->used = actualOld > memory->used ? 0 : memory->used - actualOld;
        }
        return nullptr;
    }
    const std::size_t base = memory->used >= actualOld ? memory->used - actualOld : 0;
    if (newSize > kMemoryLimit || base > kMemoryLimit - newSize) return nullptr;
    void* next = std::realloc(pointer, newSize);
    if (!next) return nullptr;
    memory->used = base + newSize;
    return next;
}

void instructionHook(lua_State* state, lua_Debug*) {
    auto* remaining = static_cast<int*>(lua_getextraspace(state));
    *remaining -= kHookInterval;
    if (*remaining <= 0) luaL_error(state, "Xzuff Lua: instruction budget exceeded");
}

bool readSource(const char* path, std::string& source) {
    std::ifstream file(path, std::ios::binary);
    if (!file) return false;
    source.assign(std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>());
    return (file.good() || file.eof()) && !source.empty() && source.size() <= kSourceLimit;
}

void openSafeLibraries(lua_State* state) {
    luaL_requiref(state, LUA_GNAME, luaopen_base, 1);
    lua_pop(state, 1);
    luaL_requiref(state, LUA_TABLIBNAME, luaopen_table, 1);
    lua_pop(state, 1);
    luaL_requiref(state, LUA_STRLIBNAME, luaopen_string, 1);
    lua_pop(state, 1);
    luaL_requiref(state, LUA_MATHLIBNAME, luaopen_math, 1);
    lua_pop(state, 1);
    luaL_requiref(state, LUA_UTF8LIBNAME, luaopen_utf8, 1);
    lua_pop(state, 1);
    luaL_requiref(state, LUA_COLIBNAME, luaopen_coroutine, 1);
    lua_pop(state, 1);

    // Scripts cannot load other files or compile arbitrary chunks through base helpers.
    for (const char* name : {"dofile", "loadfile", "load"}) {
        lua_pushnil(state);
        lua_setglobal(state, name);
    }
}

int run(const char* path) {
    std::string source;
    if (!readSource(path, source)) {
        std::cerr << "Xzuff Lua: cannot read source or source exceeds 4 MiB.\n";
        return 1;
    }

    MemoryState memory;
    // Lua 5.5 requires an explicit hash/randomization seed.
    lua_State* state = lua_newstate(limitedAllocator, &memory, 0);
    if (!state) {
        std::cerr << "Xzuff Lua: could not create VM within 16 MiB memory limit.\n";
        return 1;
    }

    *static_cast<int*>(lua_getextraspace(state)) = kInstructionLimit;
    openSafeLibraries(state);
    lua_sethook(state, instructionHook, LUA_MASKCOUNT, kHookInterval);

    const int loadStatus = luaL_loadbufferx(state, source.data(), source.size(), path, "t");
    if (loadStatus != LUA_OK) {
        const char* message = lua_tostring(state, -1);
        std::cerr << "Xzuff Lua: load error: " << (message ? message : "unknown error") << '\n';
        lua_close(state);
        return 1;
    }

    const int callStatus = lua_pcall(state, 0, LUA_MULTRET, 0);
    if (callStatus != LUA_OK) {
        const char* message = lua_tostring(state, -1);
        std::cerr << "Xzuff Lua: script error: " << (message ? message : "unknown error") << '\n';
        lua_close(state);
        return 1;
    }

    std::cout << "Xzuff Lua: script finished.\n";
    lua_close(state);
    return 0;
}
} // namespace

int main(int argc, char** argv) {
    if (argc != 2) {
        std::cerr << "Xzuff Lua Runtime 0.1.0\nUsage: " << argv[0] << " <script.luap|script.lua>\n";
        return 2;
    }
    const std::string_view extension = std::string_view(argv[1]).substr(
        std::string_view(argv[1]).find_last_of('.') == std::string_view::npos
            ? std::string_view(argv[1]).size()
            : std::string_view(argv[1]).find_last_of('.'));
    if (extension != ".luap" && extension != ".lua") {
        std::cerr << "Xzuff Lua: expected .luap or .lua source.\n";
        return 2;
    }
    return run(argv[1]);
}
