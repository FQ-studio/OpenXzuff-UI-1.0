#include "xzfg/gravity_kernel_backend_rv64.hpp"

namespace {

inline long gravity_ecall1(long number, long arg0) noexcept {
    register long a0 asm("a0") = arg0;
    register long a7 asm("a7") = number;
    asm volatile("ecall" : "+r"(a0) : "r"(a7) : "memory");
    return a0;
}

inline long gravity_ecall2(long number, long arg0, long arg1) noexcept {
    register long a0 asm("a0") = arg0;
    register long a1 asm("a1") = arg1;
    register long a7 asm("a7") = number;
    asm volatile("ecall" : "+r"(a0) : "r"(a1), "r"(a7) : "memory");
    return a0;
}

inline long gravity_ecall3(long number, long arg0, long arg1,
                           long arg2) noexcept {
    register long a0 asm("a0") = arg0;
    register long a1 asm("a1") = arg1;
    register long a2 asm("a2") = arg2;
    register long a7 asm("a7") = number;
    asm volatile("ecall" : "+r"(a0) : "r"(a1), "r"(a2), "r"(a7) : "memory");
    return a0;
}

xzfg_status task_yield(void *) noexcept {
    return static_cast<xzfg_status>(
        gravity_ecall1(XZFG_SYSCALL_YIELD, 0));
}

xzfg_status task_exit(void *, int32_t status) noexcept {
    /* Successful Exit never resumes this task; a return is an error path. */
    return static_cast<xzfg_status>(
        gravity_ecall1(XZFG_SYSCALL_EXIT, static_cast<long>(status)));
}

xzfg_status memory_map(void *, uint64_t hint, uint64_t length,
                       uint32_t protection, uint64_t *out_address) noexcept {
    constexpr uint32_t known = XZFG_MEMORY_READ | XZFG_MEMORY_WRITE |
                               XZFG_MEMORY_EXECUTE | XZFG_MEMORY_USER;
    if (out_address == nullptr || length == 0 || (protection & ~known) != 0) {
        return XZFG_STATUS_INVALID_ARGUMENT;
    }
    const long result = gravity_ecall3(
        XZFG_SYSCALL_MAP_MEMORY, static_cast<long>(hint),
        static_cast<long>(length), static_cast<long>(protection));
    if (result < 0) return static_cast< xzfg_status >(result);
    *out_address = static_cast<uint64_t>(result);
    return XZFG_STATUS_OK;
}

xzfg_status memory_unmap(void *, uint64_t address, uint64_t length) noexcept {
    if (address == 0 || length == 0) return XZFG_STATUS_INVALID_ARGUMENT;
    return static_cast<xzfg_status>(gravity_ecall2(
        XZFG_SYSCALL_UNMAP_MEMORY, static_cast<long>(address),
        static_cast<long>(length)));
}

xzfg_status memory_protect(void *, uint64_t address, uint64_t length,
                           uint32_t protection) noexcept {
    constexpr uint32_t known = XZFG_MEMORY_READ | XZFG_MEMORY_WRITE |
                               XZFG_MEMORY_EXECUTE | XZFG_MEMORY_USER;
    if (address == 0 || length == 0 || (protection & ~known) != 0) {
        return XZFG_STATUS_INVALID_ARGUMENT;
    }
    return static_cast<xzfg_status>(gravity_ecall3(
        XZFG_SYSCALL_PROTECT_MEMORY, static_cast<long>(address),
        static_cast<long>(length), static_cast<long>(protection)));
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
    api->memory_protect = memory_protect;
    return xzfg_gravity_host_api_v1_valid(api);
}
