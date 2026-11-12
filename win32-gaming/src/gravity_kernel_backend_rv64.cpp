#include "xzfg/gravity_kernel_backend_rv64.hpp"

namespace {

constexpr long kSyscallExit = XZFG_SYSCALL_EXIT;
constexpr long kSyscallYield = XZFG_SYSCALL_YIELD;

inline xzfg_status gravity_ecall(long number, long argument) noexcept {
    register long a0 asm("a0") = argument;
    register long a7 asm("a7") = number;
    asm volatile("ecall" : "+r"(a0) : "r"(a7) : "memory");
    return static_cast<xzfg_status>(a0);
}

xzfg_status task_yield(void *) noexcept {
    return gravity_ecall(kSyscallYield, 0);
}

xzfg_status task_exit(void *, int32_t status) noexcept {
    /* On success the kernel marks this task Exited and switches frames.
       A return is possible only when the kernel rejects the syscall. */
    return gravity_ecall(kSyscallExit, static_cast<long>(status));
}

xzfg_status memory_map(void *, uint64_t, uint64_t length,
                       uint32_t protection, uint64_t *out_address) noexcept {
    constexpr uint32_t known = XZFG_MEMORY_READ | XZFG_MEMORY_WRITE |
                               XZFG_MEMORY_EXECUTE | XZFG_MEMORY_USER;
    if (out_address == nullptr || length == 0 || (protection & ~known) != 0) {
        return XZFG_STATUS_INVALID_ARGUMENT;
    }
    return XZFG_STATUS_NOT_IMPLEMENTED;
}

xzfg_status memory_unmap(void *, uint64_t address, uint64_t length) noexcept {
    if (address == 0 || length == 0) return XZFG_STATUS_INVALID_ARGUMENT;
    return XZFG_STATUS_NOT_IMPLEMENTED;
}

} // namespace

extern "C" int xzfg_gravity_host_api_v1_bind_rv64(
    xzfg_gravity_host_api_v1 *api) {
    if (api == nullptr) return 0;
    *api = {};
    api->abi_version = XZFG_GRAVITY_HOST_ABI_VERSION;
    api->struct_size = static_cast<uint32_t>(sizeof(*api));
    api->context = nullptr;
    api->task_yield = task_yield;
    api->task_exit = task_exit;
    api->memory_map = memory_map;
    api->memory_unmap = memory_unmap;
    return xzfg_gravity_host_api_v1_valid(api);
}
