#ifndef GRAVITY_FDT_H
#define GRAVITY_FDT_H

#include <stddef.h>
#include "gravity/memory_map.h"

/*
 * Parse a bootloader-provided Flattened Device Tree (FDT) and add its RAM
 * nodes to map. The parser validates all offsets against available_bytes,
 * supports one- or two-cell physical addresses/sizes, and removes FDT
 * memreserve entries from usable RAM. Kernel image and stack reservations
 * remain the caller's responsibility.
 *
 * Returns 0 when at least one RAM range was parsed; otherwise -1.
 */
int gravity_fdt_add_memory(const void *blob, size_t available_bytes,
                           gravity_memory_map_t *map, size_t *parsed_total_size);

#endif
