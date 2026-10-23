#ifndef GRAVITY_MEMORY_MAP_H
#define GRAVITY_MEMORY_MAP_H

#include <stddef.h>
#include <stdint.h>

#define GRAVITY_MEMORY_MAP_MAX_REGIONS 32u

typedef struct gravity_memory_region {
    uintptr_t start; /* inclusive */
    uintptr_t end;   /* exclusive */
} gravity_memory_region_t;

typedef struct gravity_memory_map {
    gravity_memory_region_t regions[GRAVITY_MEMORY_MAP_MAX_REGIONS];
    size_t count;
} gravity_memory_map_t;

/* Initialize an empty map. */
void gravity_memory_map_init(gravity_memory_map_t *map);

/* Add a non-empty, non-overlapping usable RAM range. End is exclusive. */
int gravity_memory_map_add(gravity_memory_map_t *map,
                           uintptr_t start, uintptr_t end);

/*
 * Remove [start,end) from usable RAM, splitting ranges when needed.
 * Returns 0 when the whole reservation was already outside RAM or removed;
 * returns -1 for invalid input or insufficient map capacity.
 */
int gravity_memory_map_reserve(gravity_memory_map_t *map,
                               uintptr_t start, uintptr_t end);

/* Total usable bytes, saturating at SIZE_MAX if sum overflows. */
size_t gravity_memory_map_usable_bytes(const gravity_memory_map_t *map);

#endif
