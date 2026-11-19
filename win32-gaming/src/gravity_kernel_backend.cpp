#include "xzfg/gravity_kernel_backend.hpp"

#include "openxzuff/kernel/contracts.hpp"

#include <cstdint>

namespace {

using openxzuff::kernel::SyscallArguments;
using openxzuff::kernel::SyscallHooks;
using openxzuff::kernel::dispatch_syscall;

xzfg_status gravity_yield(void *context) {
    if (context == nullptr) return XZFG_STATUS_INVALID_ARGUMENT;
    const SyscallArguments arguments{};
    const auto outcome = dispatch_syscall(
        static_cast<std::uint64_t>(openxzuff::kernel::SyscallId::Yield),
        arguments, *static_cast<SyscallHooks *>(context));
    return static_cast<xzfg_status>(outcome.value);
}

xzfg_status gravity_exit(void *context, int32_t exit_status) {
    if (context == nullptr) return XZFG_STATUS_INVALID_ARGUMENT;
    SyscallArguments arguments{};
    arguments.values[0] = static_cast<std::uint64_t>(
        static_cast<std::int64_t>(exit_status));
    const auto outcome = dispatch_syscall(
        static_cast<std::uint64_t>(openxzuff::kernel::SyscallId::Exit),
        arguments, *static_cast<SyscallHooks *>(context));
    if (outcome.terminated) return XZFG_STATUS_OK;
    return static_cast<xzfg_status>(outcome.value);
}

xzfg_status gravity_memory_map(void *, uint64_t, uint64_t length,
                               uint32_t protection, uint64_t *out_address) {
    constexpr uint32_t known = XZFG_MEMORY_READ | XZFG_MEMORY_WRITE |
                               XZFG_MEMORY_EXECUTE | XZFG_MEMORY_USER;
    if (out_address == nullptr || length == 0 ||
        (protection & ~known) != 0) {
        return XZFG_STATUS_INVALID_ARGUMENT;
    }
    return XZFG_STATUS_NOT_IMPLEMENTED;
}

xzfg_status gravity_memory_unmap(void *, uint64_t address, uint64_t length) {
    if (address == 0 || length == 0) return XZFG_STATUS_INVALID_ARGUMENT;
    return XZFG_STATUS_NOT_IMPLEMENTED;
}

} // namespace

namespace xzfg {

bool gravity_host_api_v1_bind_syscall_hooks(
    xzfg_gravity_host_api_v1 *api, SyscallHooks *hooks) noexcept {
    if (api == nullptr || hooks == nullptr) return false;

    *api = {};
    api->abi_version = XZFG_GRAVITY_HOST_ABI_VERSION;
    api->struct_size = static_cast<uint32_t>(sizeof(*api));
    api->context = hooks;
    api->task_yield = gravity_yield;
    api->task_exit = gravity_exit;
    api->memory_map = gravity_memory_map;
    api->memory_unmap = gravity_memory_unmap;

    return xzfg_gravity_host_api_v1_valid(api) != 0;
}

} // namespace xzfg
