#ifndef XZFG_GRAVITY_HOST_ABI_H
#define XZFG_GRAVITY_HOST_ABI_H

/* Stable C ABI boundary for the OpenXzuff Win32 Gaming host adapter.
 * This is a contract, not a claim that every Gravity service is implemented.
 * ABI v1 deliberately exposes only operations that can be backed by the
 * existing process/syscall direction without inventing syscall numbers.
 */
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define XZFG_GRAVITY_HOST_ABI_VERSION 1u

typedef uint64_t xzfg_handle;
typedef int32_t xzfg_status;

enum {
    XZFG_STATUS_OK = 0,
    XZFG_STATUS_INVALID_ARGUMENT = -22,
    XZFG_STATUS_BAD_ADDRESS = -14,
    XZFG_STATUS_NOT_IMPLEMENTED = -38,
    XZFG_STATUS_NO_MEMORY = -12,
    XZFG_STATUS_WOULD_BLOCK = -11
};

enum {
    XZFG_MEMORY_READ = 1u << 0,
    XZFG_MEMORY_WRITE = 1u << 1,
    XZFG_MEMORY_EXECUTE = 1u << 2,
    XZFG_MEMORY_USER = 1u << 3
};

enum {
    XZFG_WAIT_ANY = 0u,
    XZFG_WAIT_ALL = 1u
};

typedef struct xzfg_gravity_host_api_v1 {
    uint32_t abi_version;
    uint32_t struct_size;
    void *context;

    /* Process/thread lifecycle. Entry points are native RV64 callbacks;
       translated x86/x64 execution is a later subsystem. */
    xzfg_status (*task_yield)(void *context);
    xzfg_status (*task_exit)(void *context, int32_t exit_status);
    xzfg_status (*task_create)(void *context, uint64_t entry,
                               uint64_t stack_top, xzfg_handle *out_task);

    /* Address-space operations. Protection bits are XZFG_MEMORY_* values. */
    xzfg_status (*memory_map)(void *context, uint64_t hint, uint64_t length,
                              uint32_t protection, uint64_t *out_address);
    xzfg_status (*memory_unmap)(void *context, uint64_t address,
                                uint64_t length);
    xzfg_status (*memory_protect)(void *context, uint64_t address,
                                  uint64_t length, uint32_t protection);

    /* Opaque OpenXzuff VFS/service handles; never Linux file descriptors. */
    xzfg_status (*handle_read)(void *context, xzfg_handle handle,
                               void *buffer, uint64_t length,
                               uint64_t *out_read);
    xzfg_status (*handle_write)(void *context, xzfg_handle handle,
                                const void *buffer, uint64_t length,
                                uint64_t *out_written);
    xzfg_status (*handle_close)(void *context, xzfg_handle handle);

    /* Waitable Gravity objects; replaces poll/epoll/eventfd semantics. */
    xzfg_status (*wait_handles)(void *context, const xzfg_handle *handles,
                                uint32_t count, uint32_t mode,
                                uint64_t timeout_ticks, uint32_t *out_index);
} xzfg_gravity_host_api_v1;

/* Validate only ABI shape and mandatory v1 operations. Optional service
 * callbacks may be null and must return NOT_IMPLEMENTED at the adapter layer. */
static inline int xzfg_gravity_host_api_v1_valid(
    const xzfg_gravity_host_api_v1 *api) {
    return api != NULL &&
           api->abi_version == XZFG_GRAVITY_HOST_ABI_VERSION &&
           api->struct_size >= (uint32_t)sizeof(xzfg_gravity_host_api_v1) &&
           api->task_yield != NULL &&
           api->task_exit != NULL &&
           api->memory_map != NULL &&
           api->memory_unmap != NULL;
}

#ifdef __cplusplus
}
#endif

#endif
