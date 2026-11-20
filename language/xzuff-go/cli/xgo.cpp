#include <cerrno>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <string>
#include <string_view>
#include <vector>

#ifdef _WIN32
#include <process.h>
#else
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>
#endif

namespace fs = std::filesystem;

namespace {
fs::path executableDir(const char* argv0) {
    std::error_code ec;
    const fs::path p = fs::absolute(argv0, ec);
    return ec ? fs::current_path() : p.parent_path();
}

// Launch sibling tools without a shell. This keeps paths with spaces safe and
// avoids depending on POSIX shell quoting when xgo is built for Windows.
int invoke(const fs::path& executable, const std::vector<fs::path>& args) {
    std::vector<std::string> storage;
    storage.reserve(args.size() + 1);
    storage.push_back(executable.string());
    for (const auto& arg : args) storage.push_back(arg.string());

#ifdef _WIN32
    std::vector<const char*> argv;
#else
    std::vector<char*> argv;
#endif
    argv.reserve(storage.size() + 1);
    for (auto& value : storage) argv.push_back(value.data());
    argv.push_back(nullptr);

#ifdef _WIN32
    const intptr_t result = _spawnv(_P_WAIT, storage.front().c_str(), argv.data());
    if (result == -1) {
        std::cerr << "xgo: cannot start " << executable << ": "
                  << std::strerror(errno) << '\\n';
        return 127;
    }
    return static_cast<int>(result);
#else
    const pid_t child = fork();
    if (child < 0) {
        std::cerr << "xgo: fork failed: " << std::strerror(errno) << '\\n';
        return 127;
    }
    if (child == 0) {
        execv(storage.front().c_str(), argv.data());
        _exit(errno == ENOENT ? 127 : 126);
    }

    int status = 0;
    while (waitpid(child, &status, 0) < 0) {
        if (errno == EINTR) continue;
        std::cerr << "xgo: waitpid failed: " << std::strerror(errno) << '\\n';
        return 127;
    }
    if (WIFEXITED(status)) return WEXITSTATUS(status);
    if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);
    return 1;
#endif
}

bool validName(std::string_view name) {
    if (name.empty() || name == "." || name == "..") return false;
    for (unsigned char c : name) {
        if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
              (c >= '0' && c <= '9') || c == '-' || c == '_')) return false;
    }
    return true;
}

bool hasProject(const fs::path& root) {
    return fs::is_regular_file(root / "xgo.project") &&
           fs::is_regular_file(root / "src" / "main.xgo");
}

int usage(bool toError = true) {
    auto& out = toError ? std::cerr : std::cout;
    out << "Xzuff Go CLI\n"
        << "  xgo create <nama_projek>\n"
        << "  xgo check\n"
        << "  xgo run\n"
        << "  xgo build\n"
        << "  xgo delete <nama_projek>\n";
    return toError ? 2 : 0;
}

int requireProject(const fs::path& root, std::string_view command) {
    if (hasProject(root)) return 0;
    std::cerr << "xgo " << command
              << ": current directory is not an Xzuff Go project "
                 "(expected xgo.project and src/main.xgo)\n";
    return 1;
}
}

int main(int argc, char** argv) {
    if (argc == 2 && (std::string_view(argv[1]) == "--help" ||
                      std::string_view(argv[1]) == "help")) return usage(false);
    if (argc < 2) return usage();

    const std::string_view command(argv[1]);
    const fs::path tools = executableDir(argv[0]);
    const fs::path checker = tools / "xzuff-check";
    const fs::path compiler = tools / "xzuff-go";
    const fs::path root = fs::current_path();

    if (command == "create") {
        if (argc != 3 || !validName(argv[2])) return usage();
        const fs::path target = (root / argv[2]).lexically_normal();
        if (target.parent_path() != root) {
            std::cerr << "xgo create: project must be created directly inside the current directory\n";
            return 1;
        }
        if (fs::exists(target)) {
            std::cerr << "xgo create: project already exists: " << target << '\n';
            return 1;
        }

        std::error_code ec;
        fs::create_directories(target / "src", ec);
        if (!ec) fs::create_directories(target / "tests", ec);
        if (ec) {
            std::cerr << "xgo create: cannot create project: " << ec.message() << '\n';
            return 1;
        }
        {
            std::ofstream manifest(target / "xgo.project");
            manifest << "name = \"" << argv[2] << "\"\n"
                     << "source = \"src/main.xgo\"\n"
                     << "tests = \"tests\"\n";
            if (!manifest) {
                std::cerr << "xgo create: failed writing project manifest\n";
                return 1;
            }
        }
        {
            std::ofstream source(target / "src" / "main.xgo");
            source << "function main() returns i32 {\n    return 0;\n}\n";
            if (!source) {
                std::cerr << "xgo create: failed writing starter source\n";
                return 1;
            }
        }
        {
            std::ofstream test(target / "tests" / "smoke_test.xgo");
            test << "// Source smoke test; this milestone checks syntax only.\n"
                 << "function main() returns i32 {\n    return 0;\n}\n";
            if (!test) {
                std::cerr << "xgo create: failed writing smoke test\n";
                return 1;
            }
        }
        std::cout << "Created Xzuff Go project: " << target << '\n';
        return 0;
    }

    if (command == "check") {
        if (argc != 2) return usage();
        if (requireProject(root, command) != 0) return 1;
        if (!fs::is_regular_file(checker)) {
            std::cerr << "xgo check: Coder Checker executable not found: " << checker << '\n';
            return 1;
        }
        std::cout << "Coder Checker: checking src/main.xgo\n";
        const int result = invoke(checker, {root / "src" / "main.xgo"});
        if (result != 0) {
            std::cerr << "xgo check: FAIL (checker exit " << result << ")\n";
            return result == 2 ? 1 : result;
        }
        std::cout << "xgo check: PASS\n";
        return 0;
    }

    if (command == "build") {
        if (argc != 2) return usage();
        if (requireProject(root, command) != 0) return 1;
        if (!fs::is_regular_file(checker) || !fs::is_regular_file(compiler)) {
            std::cerr << "xgo build: compiler or Coder Checker executable is missing from " << tools << '\n';
            return 1;
        }
        std::cout << "xgo build: running Coder Checker first\n";
        if (invoke(checker, {root / "src" / "main.xgo"}) != 0) {
            std::cerr << "xgo build: stopped because Coder Checker failed\n";
            return 1;
        }
        const fs::path output = root / "build" / (root.filename().string() + ".xexe");
        std::error_code ec;
        fs::create_directories(output.parent_path(), ec);
        if (ec) {
            std::cerr << "xgo build: cannot create build directory: " << ec.message() << '\n';
            return 1;
        }
        if (invoke(compiler, {root / "src" / "main.xgo", output}) != 0) {
            std::cerr << "xgo build: compiler failed\n";
            return 1;
        }
        std::cout << "xgo build: PASS (" << output << ")\n";
        return 0;
    }

    if (command == "run") {
        if (argc != 2) return usage();
        if (requireProject(root, command) != 0) return 1;
        if (!fs::is_regular_file(checker)) {
            std::cerr << "xgo run: Coder Checker executable not found: " << checker << '\n';
            return 1;
        }
        const fs::path tests = root / "tests";
        if (!fs::is_directory(tests)) {
            std::cerr << "xgo run: tests directory not found\n";
            return 1;
        }
        std::size_t count = 0;
        for (const auto& entry : fs::directory_iterator(tests)) {
            if (!entry.is_regular_file() || entry.path().extension() != ".xgo") continue;
            ++count;
            std::cout << "[TEST] " << entry.path().filename().string() << '\n';
            const int result = invoke(checker, {entry.path()});
            if (result != 0) {
                std::cerr << "[FAIL] " << entry.path().filename().string()
                          << " (checker exit " << result << ")\n";
                return 1;
            }
            std::cout << "[PASS] " << entry.path().filename().string() << '\n';
        }
        if (count == 0) {
            std::cerr << "xgo run: no .xgo test files found in tests/\n";
            return 1;
        }
        std::cout << "xgo run: PASS (" << count
                  << " source smoke test(s)); no build was performed\n";
        return 0;
    }

    if (command == "delete") {
        if (argc != 3 || !validName(argv[2])) return usage();
        const fs::path target = (root / argv[2]).lexically_normal();
        if (target.parent_path() != root || !hasProject(target)) {
            std::cerr << "xgo delete: target is not a project directly inside the current directory\n";
            return 1;
        }
        std::cout << "Delete project '" << target.filename().string() << "' at "
                  << target << "? Type DELETE to confirm: " << std::flush;
        std::string answer;
        if (!std::getline(std::cin, answer) || answer != "DELETE") {
            std::cout << "xgo delete: cancelled\n";
            return 1;
        }
        std::error_code ec;
        fs::remove_all(target, ec);
        if (ec) {
            std::cerr << "xgo delete: failed: " << ec.message() << '\n';
            return 1;
        }
        std::cout << "xgo delete: project deleted\n";
        return 0;
    }

    return usage();
}
