#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <string>
#include <string_view>
#include <vector>

namespace fs = std::filesystem;

namespace {
fs::path executableDir(const char* argv0) {
    std::error_code ec;
    fs::path p = fs::absolute(argv0, ec);
    if (ec) return fs::current_path();
    return p.parent_path();
}

std::string quote(const fs::path& p) {
    std::string s = p.string();
    std::string out = "\"";
    for (char c : s) {
        if (c == '\"' || c == '\\' || c == '
        out.push_back(c);
    }
    out.push_back('\"');
    return out;
}

int invoke(const fs::path& executable, const std::vector<fs::path>& args) {
    std::string command = quote(executable);
    for (const auto& arg : args) command += " " + quote(arg);
    return std::system(command.c_str());
}

bool validName(std::string_view name) {
    if (name.empty() || name == "." || name == "..") return false;
    for (unsigned char c : name) {
        if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
              (c >= '0' && c <= '9') || c == '-' || c == '_')) return false;
    }
    return true;
}

fs::path projectRoot() { return fs::current_path(); }

bool hasProject(const fs::path& root) {
    return fs::is_regular_file(root / "xgo.project") &&
           fs::is_regular_file(root / "src" / "main.xgo");
}

int usage() {
    std::cerr << "Xzuff Go CLI\n"
              << "  xgo create <nama_projek>\n"
              << "  xgo check\n"
              << "  xgo run\n"
              << "  xgo build\n"
              << "  xgo delete <nama_projek>\n";
    return 2;
}
}

int main(int argc, char** argv) {
    if (argc == 2 && (std::string_view(argv[1]) == "--help" || std::string_view(argv[1]) == "help")) { usage(); return 0; }
    if (argc < 2) return usage();
    const std::string_view command(argv[1]);
    const fs::path tools = executableDir(argv[0]);
    const fs::path checker = tools / "xzuff-check";
    const fs::path compiler = tools / "xzuff-go";
    const fs::path root = projectRoot();

    if (command == "create") {
        if (argc != 3 || !validName(argv[2])) return usage();
        const fs::path target = fs::absolute(argv[2]).lexically_normal();
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
        }
        {
            std::ofstream source(target / "src" / "main.xgo");
            source << "function main() returns i32 {\n    return 0;\n}\n";
        }
        {
            std::ofstream test(target / "tests" / "smoke_test.xgo");
            test << "// Initial source smoke test; current Xzuff Go subset supports syntax-check tests only.\n"
                 << "function main() returns i32 {\n    return 0;\n}\n";
        }
        std::cout << "Created Xzuff Go project: " << target << '\n';
        return 0;
    }

    if (command == "check") {
        if (argc != 2 || !hasProject(root)) {
            std::cerr << "xgo check: current directory is not an Xzuff Go project (expected xgo.project and src/main.xgo)\n";
            return argc == 2 ? 1 : usage();
        }
        std::cout << "Coder Checker: checking src/main.xgo\n";
        const int result = invoke(checker, {root / "src" / "main.xgo"});
        if (result != 0) {
            std::cerr << "xgo check: FAIL\n";
            return 1;
        }
        std::cout << "xgo check: PASS\n";
        return 0;
    }

    if (command == "build") {
        if (argc != 2 || !hasProject(root)) {
            std::cerr << "xgo build: current directory is not an Xzuff Go project\n";
            return argc == 2 ? 1 : usage();
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
        if (argc != 2 || !hasProject(root)) {
            std::cerr << "xgo run: current directory is not an Xzuff Go project\n";
            return argc == 2 ? 1 : usage();
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
            if (invoke(checker, {entry.path()}) != 0) {
                std::cerr << "[FAIL] " << entry.path().filename().string() << '\n';
                return 1;
            }
            std::cout << "[PASS] " << entry.path().filename().string() << '\n';
        }
        if (count == 0) {
            std::cerr << "xgo run: no .xgo test files found in tests/\n";
            return 1;
        }
        std::cout << "xgo run: PASS (" << count << " source smoke test(s)); no build was performed\n";
        return 0;
    }

    if (command == "delete") {
        if (argc != 3 || !validName(argv[2])) return usage();
        const fs::path projectsRoot = fs::current_path();
        const fs::path target = (projectsRoot / argv[2]).lexically_normal();
        if (target.parent_path() != projectsRoot || !hasProject(target)) {
            std::cerr << "xgo delete: target is not a project directly inside the current directory\n";
            return 1;
        }
        std::cout << "Delete project '" << target.filename().string() << "' at " << target << "? Type DELETE to confirm: ";
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
 || c == '\`') out.push_back('\\');
        out.push_back(c);
    }
    out.push_back('\"');
    return out;
}

int invoke(const fs::path& executable, const std::vector<fs::path>& args) {
    std::string command = quote(executable);
    for (const auto& arg : args) command += " " + quote(arg);
    return std::system(command.c_str());
}

bool validName(std::string_view name) {
    if (name.empty() || name == "." || name == "..") return false;
    for (unsigned char c : name) {
        if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
              (c >= '0' && c <= '9') || c == '-' || c == '_')) return false;
    }
    return true;
}

fs::path projectRoot() { return fs::current_path(); }

bool hasProject(const fs::path& root) {
    return fs::is_regular_file(root / "xgo.project") &&
           fs::is_regular_file(root / "src" / "main.xgo");
}

int usage() {
    std::cerr << "Xzuff Go CLI\n"
              << "  xgo create <nama_projek>\n"
              << "  xgo check\n"
              << "  xgo run\n"
              << "  xgo build\n"
              << "  xgo delete <nama_projek>\n";
    return 2;
}
}

int main(int argc, char** argv) {
    if (argc == 2 && (std::string_view(argv[1]) == "--help" || std::string_view(argv[1]) == "help")) { usage(); return 0; }
    if (argc < 2) return usage();
    const std::string_view command(argv[1]);
    const fs::path tools = executableDir(argv[0]);
    const fs::path checker = tools / "xzuff-check";
    const fs::path compiler = tools / "xzuff-go";
    const fs::path root = projectRoot();

    if (command == "create") {
        if (argc != 3 || !validName(argv[2])) return usage();
        const fs::path target = fs::absolute(argv[2]).lexically_normal();
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
        }
        {
            std::ofstream source(target / "src" / "main.xgo");
            source << "function main() returns i32 {\n    return 0;\n}\n";
        }
        {
            std::ofstream test(target / "tests" / "smoke_test.xgo");
            test << "// Initial source smoke test; current Xzuff Go subset supports syntax-check tests only.\n"
                 << "function main() returns i32 {\n    return 0;\n}\n";
        }
        std::cout << "Created Xzuff Go project: " << target << '\n';
        return 0;
    }

    if (command == "check") {
        if (argc != 2 || !hasProject(root)) {
            std::cerr << "xgo check: current directory is not an Xzuff Go project (expected xgo.project and src/main.xgo)\n";
            return argc == 2 ? 1 : usage();
        }
        std::cout << "Coder Checker: checking src/main.xgo\n";
        const int result = invoke(checker, {root / "src" / "main.xgo"});
        if (result != 0) {
            std::cerr << "xgo check: FAIL\n";
            return 1;
        }
        std::cout << "xgo check: PASS\n";
        return 0;
    }

    if (command == "build") {
        if (argc != 2 || !hasProject(root)) {
            std::cerr << "xgo build: current directory is not an Xzuff Go project\n";
            return argc == 2 ? 1 : usage();
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
        if (argc != 2 || !hasProject(root)) {
            std::cerr << "xgo run: current directory is not an Xzuff Go project\n";
            return argc == 2 ? 1 : usage();
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
            if (invoke(checker, {entry.path()}) != 0) {
                std::cerr << "[FAIL] " << entry.path().filename().string() << '\n';
                return 1;
            }
            std::cout << "[PASS] " << entry.path().filename().string() << '\n';
        }
        if (count == 0) {
            std::cerr << "xgo run: no .xgo test files found in tests/\n";
            return 1;
        }
        std::cout << "xgo run: PASS (" << count << " source smoke test(s)); no build was performed\n";
        return 0;
    }

    if (command == "delete") {
        if (argc != 3 || !validName(argv[2])) return usage();
        const fs::path projectsRoot = fs::current_path();
        const fs::path target = (projectsRoot / argv[2]).lexically_normal();
        if (target.parent_path() != projectsRoot || !hasProject(target)) {
            std::cerr << "xgo delete: target is not a project directly inside the current directory\n";
            return 1;
        }
        std::cout << "Delete project '" << target.filename().string() << "' at " << target << "? Type DELETE to confirm: ";
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
