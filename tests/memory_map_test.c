#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include "gravity/memory_map.h"

int main(void)
{
    gravity_memory_map_t map;
    gravity_memory_map_init(&map);

    assert(gravity_memory_map_add(&map, 0x1000, 0x9000) == 0);
    assert(gravity_memory_map_add(&map, 0xa000, 0xc000) == 0);
    assert(map.count == 2);
    assert(map.regions[0].start == 0x1000);
    assert(gravity_memory_map_add(&map, 0x2000, 0x3000) == -1);
    assert(gravity_memory_map_add(&map, 0x4000, 0x4000) == -1);

    /* Simulate reserving kernel image + stack, then an FDT blob. */
    assert(gravity_memory_map_reserve(&map, 0x3000, 0x5000) == 0);
    assert(map.count == 3);
    assert(map.regions[0].start == 0x1000 && map.regions[0].end == 0x3000);
    assert(map.regions[1].start == 0x5000 && map.regions[1].end == 0x9000);
    assert(map.regions[2].start == 0xa000 && map.regions[2].end == 0xc000);

    assert(gravity_memory_map_reserve(&map, 0x1000, 0x2000) == 0);
    assert(map.regions[0].start == 0x2000);
    assert(gravity_memory_map_reserve(&map, 0x7000, 0xb000) == 0);
    assert(map.count == 3);
    assert(map.regions[1].start == 0x5000 && map.regions[1].end == 0x7000);
    assert(map.regions[2].start == 0xb000 && map.regions[2].end == 0xc000);

    assert(gravity_memory_map_usable_bytes(&map) == 0x4000);
    assert(gravity_memory_map_reserve(&map, 0, 1) == -1);
    assert(gravity_memory_map_usable_bytes(NULL) == 0);

    puts("Gravity physical memory map tests: PASS");
    return 0;
}
