#include "openxzuff/kernel/syscall.hpp"
#include "openxzuff/kernel/contracts.hpp"

namespace openxzuff::kernel {

SyscallOutcome dispatch_syscall(std::uint64_t syscall_id,
                                const SyscallArguments& arguments,
                                const SyscallHooks& hooks) noexcept {
    switch (static_cast<SyscallId>(syscall_id)) {
    case SyscallId::Exit:
        if (hooks.exit_task == nullptr) {
            return {syscall_error_not_implemented, false};
        }
        hooks.exit_task(hooks.context, arguments.values[0]);
        return {0, true};

    case SyscallId::Yield:
        if (hooks.yield_task == nullptr) {
            return {syscall_error_not_implemented, false};
        }
        hooks.yield_task(hooks.context);
        return {0, false};

    case SyscallId::SendMessage:
    case SyscallId::ReceiveMessage:
    case SyscallId::MapMemory:
        return {syscall_error_not_implemented, false};
    }

    return {syscall_error_not_implemented, false};
}

} // namespace openxzuff::kernel
