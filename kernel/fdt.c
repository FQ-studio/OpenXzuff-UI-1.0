#include "gravity/fdt.h"

#include <stdint.h>

#define FDT_MAGIC 0xd00dfeedu
#define FDT_BEGIN_NODE 1u
#define FDT_END_NODE 2u
#define FDT_PROP 3u
#define FDT_NOP 4u
#define FDT_END 9u
#define FDT_HEADER_SIZE 40u
#define FDT_MAX_DEPTH 32u

static uint32_t be32(const uint8_t *p)
{
    return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) |
           ((uint32_t)p[2] << 8) | (uint32_t)p[3];
}

static uint64_t read_cells(const uint8_t *p, uint32_t cells)
{
    uint64_t value = 0;
    uint32_t i;
    for (i = 0; i < cells; ++i)
        value = (value << 32) | be32(p + (size_t)i * 4u);
    return value;
}

static int name_is(const uint8_t *p, uint32_t length, const char *name)
{
    uint32_t i = 0;
    while (name[i] != '\0') {
        if (i >= length || p[i] != (uint8_t)name[i])
            return 0;
        ++i;
    }
    return length == i || (length == i + 1u && p[i] == 0);
}

static int node_is_memory(const uint8_t *name, uint32_t length)
{
    static const char prefix[] = "memory";
    uint32_t i;
    if (length < sizeof(prefix) - 1u)
        return 0;
    for (i = 0; i < sizeof(prefix) - 1u; ++i)
        if (name[i] != (uint8_t)prefix[i])
            return 0;
    return length == sizeof(prefix) - 1u || name[sizeof(prefix) - 1u] == '@';
}

static int range_inside(size_t offset, size_t length, size_t total)
{
    return offset <= total && length <= total - offset;
}

int gravity_fdt_add_memory(const void *blob, size_t available_bytes,
                           gravity_memory_map_t *map, size_t *parsed_total_size)
{
    const uint8_t *b = (const uint8_t *)blob;
    uint32_t total, off_struct, off_strings, off_reserve, size_strings, size_struct;
    uint32_t address_cells = 2, size_cells = 1;
    size_t cursor, struct_end, strings_end, reserve_cursor;
    unsigned depth = 0;
    int root_seen = 0, found_memory = 0;
    int memory_nodes[FDT_MAX_DEPTH] = {0};
    int memory_device_type[FDT_MAX_DEPTH] = {0};
    const uint8_t *reg_data[FDT_MAX_DEPTH] = {0};
    uint32_t reg_length[FDT_MAX_DEPTH] = {0};
    int reserved_container[FDT_MAX_DEPTH] = {0};
    int reserved_child[FDT_MAX_DEPTH] = {0};
    uintptr_t reserved_starts[GRAVITY_MEMORY_MAP_MAX_REGIONS];
    uintptr_t reserved_ends[GRAVITY_MEMORY_MAP_MAX_REGIONS];
    size_t reserved_count = 0;

    if (parsed_total_size != NULL) *parsed_total_size = 0;
    if (b == NULL || map == NULL || available_bytes < FDT_HEADER_SIZE ||
        be32(b) != FDT_MAGIC)
        return -1;

    total = be32(b + 4);
    off_struct = be32(b + 8);
    off_strings = be32(b + 12);
    off_reserve = be32(b + 16);
    size_strings = be32(b + 32);
    size_struct = be32(b + 36);
    if (total < FDT_HEADER_SIZE || total > available_bytes ||
        !range_inside(off_struct, size_struct, total) ||
        !range_inside(off_strings, size_strings, total) ||
        off_reserve < FDT_HEADER_SIZE || off_reserve >= total)
        return -1;

    struct_end = (size_t)off_struct + size_struct;
    strings_end = (size_t)off_strings + size_strings;
    cursor = off_struct;

    /* Pass 1: root address/size cells, whose values define memory reg tuples. */
    while (cursor + 4u <= struct_end) {
        uint32_t token = be32(b + cursor);
        cursor += 4u;
        if (token == FDT_BEGIN_NODE) {
            size_t start = cursor;
            while (cursor < struct_end && b[cursor] != 0) ++cursor;
            if (cursor == struct_end) return -1;
            if (depth == 0) root_seen = 1;
            ++cursor;
            cursor = (cursor + 3u) & ~(size_t)3u;
            if (cursor > struct_end || depth >= FDT_MAX_DEPTH) return -1;
            ++depth;
            (void)start;
        } else if (token == FDT_END_NODE) {
            if (depth == 0) return -1;
            --depth;
        } else if (token == FDT_PROP) {
            uint32_t len, nameoff;
            const uint8_t *value;
            const uint8_t *propname;
            size_t n;
            if (cursor + 8u > struct_end || depth == 0) return -1;
            len = be32(b + cursor);
            nameoff = be32(b + cursor + 4u);
            cursor += 8u;
            if (!range_inside(cursor, len, struct_end) || nameoff >= size_strings)
                return -1;
            value = b + cursor;
            propname = b + off_strings + nameoff;
            n = nameoff;
            while (n < strings_end - off_strings && propname[n - nameoff] != 0) ++n;
            if (n == strings_end - off_strings) return -1;
            if (depth == 1 && name_is(propname, (uint32_t)(n - nameoff), "#address-cells")) {
                if (len != 4u) return -1;
                address_cells = be32(value);
            } else if (depth == 1 && name_is(propname, (uint32_t)(n - nameoff), "#size-cells")) {
                if (len != 4u) return -1;
                size_cells = be32(value);
            }
            cursor = (cursor + len + 3u) & ~(size_t)3u;
            if (cursor > struct_end) return -1;
        } else if (token == FDT_NOP) {
            continue;
        } else if (token == FDT_END) {
            break;
        } else return -1;
    }
    if (!root_seen || address_cells < 1 || address_cells > 2 ||
        size_cells < 1 || size_cells > 2)
        return -1;

    /* Pass 2: parse memory nodes using root cell sizes. */
    cursor = off_struct;
    depth = 0;
    while (cursor + 4u <= struct_end) {
        uint32_t token = be32(b + cursor);
        cursor += 4u;
        if (token == FDT_BEGIN_NODE) {
            size_t start = cursor;
            uint32_t len;
            while (cursor < struct_end && b[cursor] != 0) ++cursor;
            if (cursor == struct_end || depth >= FDT_MAX_DEPTH) return -1;
            len = (uint32_t)(cursor - start);
            memory_nodes[depth] = node_is_memory(b + start, len);
            reserved_container[depth] = name_is(b + start, len, "reserved-memory");
            reserved_child[depth] = depth > 0 && reserved_container[depth - 1u];
            memory_device_type[depth] = 0;
            reg_data[depth] = NULL;
            reg_length[depth] = 0;
            ++cursor;
            cursor = (cursor + 3u) & ~(size_t)3u;
            if (cursor > struct_end) return -1;
            ++depth;
        } else if (token == FDT_END_NODE) {
            if (depth == 0) return -1;
            --depth;
            if (reserved_child[depth] && reg_data[depth] != NULL && reg_length[depth] != 0) {
                uint32_t tuple_cells = address_cells + size_cells;
                uint32_t tuple_bytes = tuple_cells * 4u;
                uint32_t pos;
                if (tuple_bytes == 0 || reg_length[depth] % tuple_bytes != 0) return -1;
                for (pos = 0; pos < reg_length[depth]; pos += tuple_bytes) {
                    uint64_t base = read_cells(reg_data[depth] + pos, address_cells);
                    uint64_t size = read_cells(reg_data[depth] + pos + address_cells * 4u, size_cells);
                    if (size == 0 || base > UINTPTR_MAX || size > UINTPTR_MAX ||
                        base + size < base || base + size > UINTPTR_MAX ||
                        reserved_count >= GRAVITY_MEMORY_MAP_MAX_REGIONS) return -1;
                    reserved_starts[reserved_count] = (uintptr_t)base;
                    reserved_ends[reserved_count] = (uintptr_t)(base + size);
                    ++reserved_count;
                }
            }
            if (memory_nodes[depth] || memory_device_type[depth]) {
                uint32_t tuple_cells = address_cells + size_cells;
                uint32_t tuple_bytes = tuple_cells * 4u;
                uint32_t pos;
                if (reg_data[depth] == NULL || tuple_bytes == 0 ||
                    reg_length[depth] == 0 || reg_length[depth] % tuple_bytes != 0)
                    continue;
                for (pos = 0; pos < reg_length[depth]; pos += tuple_bytes) {
                    uint64_t base = read_cells(reg_data[depth] + pos, address_cells);
                    uint64_t size = read_cells(reg_data[depth] + pos + address_cells * 4u, size_cells);
                    if (size == 0 || base > UINTPTR_MAX || size > UINTPTR_MAX ||
                        base + size < base || base + size > UINTPTR_MAX)
                        return -1;
                    if (gravity_memory_map_add(map, (uintptr_t)base,
                                               (uintptr_t)(base + size)) != 0)
                        return -1;
                    found_memory = 1;
                }
            }
        } else if (token == FDT_PROP) {
            uint32_t len, nameoff;
            const uint8_t *value, *propname;
            size_t name_length;
            if (cursor + 8u > struct_end || depth == 0) return -1;
            len = be32(b + cursor);
            nameoff = be32(b + cursor + 4u);
            cursor += 8u;
            if (!range_inside(cursor, len, struct_end) || nameoff >= size_strings)
                return -1;
            value = b + cursor;
            propname = b + off_strings + nameoff;
            name_length = 0;
            while (nameoff + name_length < size_strings && propname[name_length] != 0)
                ++name_length;
            if (nameoff + name_length >= size_strings) return -1;
            if (name_is(propname, (uint32_t)name_length, "device_type") &&
                name_is(value, len, "memory"))
                memory_device_type[depth - 1u] = 1;
            else if (name_is(propname, (uint32_t)name_length, "reg")) {
                reg_data[depth - 1u] = value;
                reg_length[depth - 1u] = len;
            }
            cursor = (cursor + len + 3u) & ~(size_t)3u;
            if (cursor > struct_end) return -1;
        } else if (token == FDT_NOP) {
            continue;
        } else if (token == FDT_END) {
            break;
        } else return -1;
    }
    if (!found_memory) return -1;

    /* Exclude /reserved-memory children after all RAM nodes have been collected. */
    {
        size_t i;
        for (i = 0; i < reserved_count; ++i) {
            if (gravity_memory_map_reserve(map, reserved_starts[i], reserved_ends[i]) != 0)
                return -1;
        }
    }

    /* FDT memory-reservation block consists of big-endian 64-bit addr/size pairs. */
    reserve_cursor = off_reserve;
    while (range_inside(reserve_cursor, 16u, total)) {
        uint64_t address = ((uint64_t)be32(b + reserve_cursor) << 32) |
                           be32(b + reserve_cursor + 4u);
        uint64_t size = ((uint64_t)be32(b + reserve_cursor + 8u) << 32) |
                        be32(b + reserve_cursor + 12u);
        reserve_cursor += 16u;
        if (address == 0 && size == 0) break;
        if (size == 0 || address > UINTPTR_MAX || size > UINTPTR_MAX ||
            address + size < address || address + size > UINTPTR_MAX)
            return -1;
        if (gravity_memory_map_reserve(map, (uintptr_t)address,
                                      (uintptr_t)(address + size)) != 0)
            return -1;
    }
    if (!range_inside(reserve_cursor - 16u, 16u, total) ||
        !(be32(b + reserve_cursor - 16u) == 0 &&
          be32(b + reserve_cursor - 12u) == 0 &&
          be32(b + reserve_cursor - 8u) == 0 &&
          be32(b + reserve_cursor - 4u) == 0))
        return -1;

    if (parsed_total_size != NULL) *parsed_total_size = total;
    return 0;
}
