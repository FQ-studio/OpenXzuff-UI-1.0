#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include "gravity/boot_alloc.h"

int main(void)
{
    gravity_boot_allocator_t allocator;
    void *a;
    void *b;
    uintptr_t start = (uintptr_t)0x1003;
    uintptr_t limit = (uintptr_t)0x1100;

    assert(gravity_boot_alloc_init(&allocator, start, limit) == 0);
    assert(gravity_boot_alloc_remaining(&allocator) == (size_t)(limit - start));

    a = gravity_boot_alloc(&allocator, 8, 8);
    assert(a != NULL && ((uintptr_t)a & 7u) == 0);
    assert((uintptr_t)a >= start);
    assert(allocator.allocations == 1);

    b = gravity_boot_alloc(&allocator, 16, 16);
    assert(b != NULL && ((uintptr_t)b & 15u) == 0);
    assert((uintptr_t)b >= (uintptr_t)a + 8);
    assert(allocator.allocations == 2);

    assert(gravity_boot_alloc(&allocator, 0, 8) == NULL);
    assert(gravity_boot_alloc(&allocator, 4, 3) == NULL);
    assert(gravity_boot_alloc(&allocator, 4096, 16) == NULL);
    assert(gravity_boot_alloc_remaining(&allocator) < (size_t)(limit - start));

    assert(gravity_boot_alloc_init(&allocator, 0x2000, 0x2000) == -1);
    assert(gravity_boot_alloc_init(NULL, 0x2000, 0x3000) == -1);

    puts("Gravity boot allocator tests: PASS");
    return 0;
}
