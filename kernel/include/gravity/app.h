#ifndef GRAVITY_APP_H
#define GRAVITY_APP_H

#include <stddef.h>
#include <stdint.h>

/*
 * GAPP v1 metadata validator.
 *
 * The 48-byte on-disk header is little-endian and byte-parsed (no packed
 * structs or unaligned loads). This validates metadata only: it does not map
 * memory, change privilege levels, or execute application code.
 */
#define GRAVITY_APP_HEADER_SIZE 48u
#define GRAVITY_APP_VERSION 1u
#define GRAVITY_APP_ARCH_RV64 0x34365652u /* "RV64" */
#define GRAVITY_APP_FLAG_NONE 0u

typedef struct gravity_app_info {
    uint32_t architecture;
    uint32_t flags;
    uint64_t entry_offset;
    uint64_t image_size;
    uint64_t memory_size;
} gravity_app_info_t;

/* Returns 0 on success, otherwise a negative Gravity errno value. */
int gravity_app_validate(const void *file, size_t file_size,
                         gravity_app_info_t *out_info);

#endif
