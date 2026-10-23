#include "gravity/boot_alloc.h"

#include <stddef.h>
#include <stdint.h>

int gravity_boot_alloc_init(gravity_boot_allocator_t *allocator,
                            uintptr_t start, uintptr_t end)
{
    if (allocator == NULL || start == 0 || end <= start)
        return -1;

    allocator->next = start;
    allocator->limit = end;
    allocator->allocations = 0;
    return 0;
}

void *gravity_boot_alloc(gravity_boot_allocator_t *allocator,
                         size_t size, size_t alignment)
{
    uintptr_t current;
    uintptr_t aligned;
    uintptr_t end;

    if (allocator == NULL || size == 0 || alignment == 0 ||
        (alignment & (alignment - 1)) != 0 ||
        allocator->next > allocator->limit)
        return NULL;

    current = allocator->next;
    if (current > UINTPTR_MAX - (uintptr_t)(alignment - 1))
        return NULL;

    aligned = (current + (uintptr_t)(alignment - 1)) &
              ~((uintptr_t)alignment - 1);

    if ((uintptr_t)size > UINTPTR_MAX - aligned)
        return NULL;

    end = aligned + (uintptr_t)size;
    if (end > allocator->limit)
        return NULL;

    allocator->next = end;
    allocator->allocations++;
    return (void *)aligned;
}

size_t gravity_boot_alloc_remaining(const gravity_boot_allocator_t *allocator)
{
    if (allocator == NULL || allocator->next > allocator->limit)
        return 0;
    return (size_t)(allocator->limit - allocator->next);
}
