#include "xzfg/gravity_host_abi.h"

#include <cstdint>
#include <iostream>

namespace {
struct MockState {
    unsigned yields{};
    int32_t exit_status{};
    unsigned exits{};
    uint64_t mapped_address{0x40000000u};
    uint64_t mapped_length{};
    uint32_t protection{};
};

xzfg_status mock_yield(void *context) {
    ++static_cast<MockState *>(context)->yields;
    return XZFG_STATUS_OK;
}
xzfg_status mock_exit(void *context, int32_t status) {
    auto &state = *static_cast<MockState *>(context);
    state.exit_status = status;
    ++state.exits;
    return XZFG_STATUS_OK;
}
xzfg_status mock_map(void *context, uint64_t, uint64_t length,
                     uint32_t protection, uint64_t *out_address) {
    if (out_address == nullptr || length == 0 ||
        (protection & ~(XZFG_MEMORY_READ | XZFG_MEMORY_WRITE |
                        XZFG_MEMORY_EXECUTE | XZFG_MEMORY_USER)) != 0) {
        return XZFG_STATUS_INVALID_ARGUMENT;
    }
    auto &state = *static_cast<MockState *>(context);
    state.mapped_length = length;
    state.protection = protection;
    *out_address = state.mapped_address;
    return XZFG_STATUS_OK;
}
xzfg_status mock_unmap(void *, uint64_t address, uint64_t length) {
    return address != 0 && length != 0
        ? XZFG_STATUS_OK : XZFG_STATUS_INVALID_ARGUMENT;
}
bool check(bool value, const char *message) {
    if (!value) std::cerr << "FAIL: " << message << '\n';
    return value;
}
}

int main() {
    static_assert(XZFG_SYSCALL_EXIT == 0);
    static_assert(XZFG_SYSCALL_SEND_MESSAGE == 1);
    static_assert(XZFG_SYSCALL_RECEIVE_MESSAGE == 2);
    static_assert(XZFG_SYSCALL_MAP_MEMORY == 3);
    static_assert(XZFG_SYSCALL_YIELD == 4);
    static_assert(XZFG_SYSCALL_UNMAP_MEMORY == 7);
    static_assert(XZFG_SYSCALL_PROTECT_MEMORY == 8);
    MockState state{};
    xzfg_gravity_host_api_v1 api{};
    api.abi_version = XZFG_GRAVITY_HOST_ABI_VERSION;
    api.struct_size = sizeof(api);
    api.context = &state;
    api.task_yield = mock_yield;
    api.task_exit = mock_exit;
    api.memory_map = mock_map;
    api.memory_unmap = mock_unmap;

    bool ok = true;
    ok &= check(xzfg_gravity_host_api_v1_valid(&api), "valid v1 API accepted");
    ok &= check(!xzfg_gravity_host_api_v1_valid(nullptr), "null API rejected");

    auto bad_version = api;
    bad_version.abi_version = 99;
    ok &= check(!xzfg_gravity_host_api_v1_valid(&bad_version),
                "unknown ABI version rejected");

    auto short_struct = api;
    short_struct.struct_size = 4;
    ok &= check(!xzfg_gravity_host_api_v1_valid(&short_struct),
                "truncated ABI struct rejected");

    ok &= check(api.task_yield(api.context) == XZFG_STATUS_OK &&
                state.yields == 1, "yield routed through Gravity adapter");
    ok &= check(api.task_exit(api.context, 37) == XZFG_STATUS_OK &&
                state.exits == 1 && state.exit_status == 37,
                "exit status routed through Gravity adapter");

    uint64_t address = 0;
    ok &= check(api.memory_map(api.context, 0, 8192,
                XZFG_MEMORY_READ | XZFG_MEMORY_WRITE | XZFG_MEMORY_USER,
                &address) == XZFG_STATUS_OK &&
                address == state.mapped_address &&
                state.mapped_length == 8192,
                "memory map forwards validated request");
    ok &= check(api.memory_map(api.context, 0, 0, XZFG_MEMORY_READ,
                &address) == XZFG_STATUS_INVALID_ARGUMENT,
                "zero-length map rejected");
    ok &= check(api.memory_map(api.context, 0, 4096, 0x80000000u,
                &address) == XZFG_STATUS_INVALID_ARGUMENT,
                "unknown memory protection rejected");
    ok &= check(api.memory_map(api.context, 0, 4096, XZFG_MEMORY_READ,
                nullptr) == XZFG_STATUS_INVALID_ARGUMENT,
                "null map output rejected");
    ok &= check(api.memory_unmap(api.context, 0x40000000u, 4096) ==
                XZFG_STATUS_OK, "unmap routed through Gravity adapter");

    if (ok) std::cout << "Win32 Gravity host ABI v1 contract tests: PASS\n";
    return ok ? 0 : 1;
}
