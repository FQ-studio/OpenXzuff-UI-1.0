#include "xzfg/gravity_kernel_backend.hpp"

#include <cstdint>
#include <iostream>

namespace {

struct KernelState {
    unsigned yields{};
    unsigned exits{};
    std::uint64_t exit_status{};
};

void on_yield(void *context) noexcept {
    ++static_cast<KernelState *>(context)->yields;
}

void on_exit(void *context, std::uint64_t status) noexcept {
    auto &state = *static_cast<KernelState *>(context);
    ++state.exits;
    state.exit_status = status;
}

bool check(bool condition, const char *message) {
    if (!condition) std::cerr << "FAIL: " << message << '\n';
    return condition;
}

} // namespace

int main() {
    KernelState state{};
    openxzuff::kernel::SyscallHooks hooks{&state, on_yield, on_exit};
    xzfg_gravity_host_api_v1 api{};

    bool ok = true;
    ok &= check(xzfg::gravity_host_api_v1_bind_syscall_hooks(&api, &hooks),
                "backend binds to Gravity syscall hooks");
    ok &= check(xzfg_gravity_host_api_v1_valid(&api) != 0,
                "bound API satisfies ABI v1 shape");

    ok &= check(api.task_yield(api.context) == XZFG_STATUS_OK &&
                state.yields == 1,
                "yield reaches the existing Gravity syscall dispatcher");

    ok &= check(api.task_exit(api.context, 37) == XZFG_STATUS_OK &&
                state.exits == 1 && state.exit_status == 37,
                "exit reaches the existing Gravity syscall dispatcher");

    uint64_t address = 0;
    ok &= check(api.memory_map(api.context, 0, 4096, XZFG_MEMORY_READ,
                               &address) == XZFG_STATUS_NOT_IMPLEMENTED,
                "host policy shim has no VM hook (live RV64 adapter uses Rust VM syscall)");
    ok &= check(api.memory_unmap(api.context, 0x40000000u, 4096) ==
                    XZFG_STATUS_NOT_IMPLEMENTED,
                "host policy shim has no VM hook (live RV64 adapter uses Rust VM syscall)");
    ok &= check(api.memory_map(api.context, 0, 0, XZFG_MEMORY_READ,
                               &address) == XZFG_STATUS_INVALID_ARGUMENT,
                "invalid memory map request is rejected before dispatch");

    ok &= check(api.message_send == nullptr && api.message_receive == nullptr,
                "host-only policy shim does not fake live Rust IPC callbacks");

    openxzuff::kernel::SyscallHooks missing_hooks{};
    xzfg_gravity_host_api_v1 incomplete{};
    ok &= check(xzfg::gravity_host_api_v1_bind_syscall_hooks(
                    &incomplete, &missing_hooks),
                "backend can bind a hook table without scheduler/process hooks");
    ok &= check(incomplete.task_yield(incomplete.context) ==
                    XZFG_STATUS_NOT_IMPLEMENTED,
                "missing scheduler hook remains explicitly unsupported");
    ok &= check(incomplete.task_exit(incomplete.context, 0) ==
                    XZFG_STATUS_NOT_IMPLEMENTED,
                "missing exit hook remains explicitly unsupported");

    if (ok) std::cout << "Win32 Gravity kernel backend bridge tests: PASS\n";
    return ok ? 0 : 1;
}
