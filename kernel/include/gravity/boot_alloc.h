#ifndef GRAVITY_BOOT_ALLOC_H
#define GRAVITY_BOOT_ALLOC_H

#include <stddef.h>
#include <stdint.h>

/*
 * Early-boot monotonic allocator for a memory region explicitly supplied by
 * the platform memory-map code. It never guesses RAM size and never frees.
 * The caller must exclude the kernel image, stacks, device memory, and any
 * firmware-owned/reserved ranges before initializing it.
 */
typedef struct gravity_boot_allocator {
    uintptr_t next;
    uintptr_t limit;
    size_t allocations;
} gravity_boot_allocator_t;

/* Returns 0 on success; rejects null, empty, or reversed ranges. */
int gravity_boot_alloc_init(gravity_boot_allocator_t *allocator,
                            uintptr_t start, uintptr_t end);

/*
 * Allocate bytes with power-of-two alignment. Returns NULL on invalid
 * alignment, zero size, arithmetic overflow, or exhausted region.
 * Allocated memory is not automatically zeroed.
 */
void *gravity_boot_alloc(gravity_boot_allocator_t *allocator,
                         size_t size, size_t alignment);

size_t gravity_boot_alloc_remaining(const gravity_boot_allocator_t *allocator);

#endif
