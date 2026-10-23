#include "gravity/memory_map.h"

#include <stdint.h>

void gravity_memory_map_init(gravity_memory_map_t *map)
{
    if (map == NULL)
        return;
    map->count = 0;
}

int gravity_memory_map_add(gravity_memory_map_t *map,
                           uintptr_t start, uintptr_t end)
{
    size_t i;
    if (map == NULL || start == 0 || end <= start ||
        map->count >= GRAVITY_MEMORY_MAP_MAX_REGIONS)
        return -1;

    for (i = 0; i < map->count; ++i) {
        if (start < map->regions[i].end && end > map->regions[i].start)
            return -1;
    }

    i = map->count;
    while (i > 0 && map->regions[i - 1].start > start) {
        map->regions[i] = map->regions[i - 1];
        --i;
    }
    map->regions[i].start = start;
    map->regions[i].end = end;
    ++map->count;
    return 0;
}

int gravity_memory_map_reserve(gravity_memory_map_t *map,
                               uintptr_t start, uintptr_t end)
{
    size_t i;
    size_t resulting_count;

    if (map == NULL || start == 0 || end <= start)
        return -1;

    /* Preflight capacity so a failed reservation never partially mutates map. */
    resulting_count = map->count;
    for (i = 0; i < map->count; ++i) {
        gravity_memory_region_t region = map->regions[i];
        uintptr_t cut_start;
        uintptr_t cut_end;
        if (end <= region.start || start >= region.end)
            continue;
        cut_start = start > region.start ? start : region.start;
        cut_end = end < region.end ? end : region.end;
        if (cut_start == region.start && cut_end == region.end)
            --resulting_count;
        else if (cut_start != region.start && cut_end != region.end)
            ++resulting_count;
    }
    if (resulting_count > GRAVITY_MEMORY_MAP_MAX_REGIONS)
        return -1;

    for (i = 0; i < map->count; ++i) {
        gravity_memory_region_t region = map->regions[i];
        uintptr_t cut_start;
        uintptr_t cut_end;

        if (end <= region.start || start >= region.end)
            continue;

        cut_start = start > region.start ? start : region.start;
        cut_end = end < region.end ? end : region.end;

        if (cut_start == region.start && cut_end == region.end) {
            size_t j;
            for (j = i + 1; j < map->count; ++j)
                map->regions[j - 1] = map->regions[j];
            --map->count;
            --i;
        } else if (cut_start == region.start) {
            map->regions[i].start = cut_end;
        } else if (cut_end == region.end) {
            map->regions[i].end = cut_start;
        } else {
            size_t j;
            if (map->count >= GRAVITY_MEMORY_MAP_MAX_REGIONS)
                return -1;
            for (j = map->count; j > i + 1; --j)
                map->regions[j] = map->regions[j - 1];
            map->regions[i].end = cut_start;
            map->regions[i + 1].start = cut_end;
            map->regions[i + 1].end = region.end;
            ++map->count;
            i++;
        }
    }
    return 0;
}

size_t gravity_memory_map_usable_bytes(const gravity_memory_map_t *map)
{
    size_t total = 0;
    size_t i;
    if (map == NULL)
        return 0;

    for (i = 0; i < map->count; ++i) {
        uintptr_t bytes = map->regions[i].end - map->regions[i].start;
        if (bytes > (uintptr_t)(SIZE_MAX - total))
            return SIZE_MAX;
        total += (size_t)bytes;
    }
    return total;
}
