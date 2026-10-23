#include "gravity/app.h"
#include "gravity/syscall.h"

#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

static void put16(uint8_t *p, uint16_t v)
{
    p[0] = (uint8_t)v;
    p[1] = (uint8_t)(v >> 8);
}

static void put32(uint8_t *p, uint32_t v)
{
    p[0] = (uint8_t)v;
    p[1] = (uint8_t)(v >> 8);
    p[2] = (uint8_t)(v >> 16);
    p[3] = (uint8_t)(v >> 24);
}

static void put64(uint8_t *p, uint64_t v)
{
    put32(p, (uint32_t)v);
    put32(p + 4, (uint32_t)(v >> 32));
}

static void make_valid(uint8_t *file, size_t size)
{
    memset(file, 0, size);
    memcpy(file, "GAPP", 4);
    put16(file + 4, GRAVITY_APP_VERSION);
    put16(file + 6, GRAVITY_APP_HEADER_SIZE);
    put32(file + 8, GRAVITY_APP_ARCH_RV64);
    put32(file + 12, GRAVITY_APP_FLAG_NONE);
    put64(file + 16, 0);
    put64(file + 24, 4);
    put64(file + 32, 8);
    memcpy(file + GRAVITY_APP_HEADER_SIZE, "CODE", 4);
}

int main(void)
{
    uint8_t file[GRAVITY_APP_HEADER_SIZE + 4];
    gravity_app_info_t info;

    make_valid(file, sizeof(file));
    assert(gravity_app_validate(file, sizeof(file), &info) == 0);
    assert(info.architecture == GRAVITY_APP_ARCH_RV64);
    assert(info.entry_offset == 0 && info.image_size == 4 && info.memory_size == 8);

    assert(gravity_app_validate(NULL, sizeof(file), &info) == -GRAVITY_EINVAL);
    assert(gravity_app_validate(file, sizeof(file), NULL) == -GRAVITY_EINVAL);
    assert(gravity_app_validate(file, GRAVITY_APP_HEADER_SIZE - 1, &info) == -GRAVITY_EINVAL);

    file[0] = 'X';
    assert(gravity_app_validate(file, sizeof(file), &info) == -GRAVITY_EINVAL);
    make_valid(file, sizeof(file));
    put16(file + 4, 99);
    assert(gravity_app_validate(file, sizeof(file), &info) == -GRAVITY_EINVAL);
    make_valid(file, sizeof(file));
    put32(file + 8, 0x34365836u);
    assert(gravity_app_validate(file, sizeof(file), &info) == -GRAVITY_EINVAL);
    make_valid(file, sizeof(file));
    put64(file + 16, 4);
    assert(gravity_app_validate(file, sizeof(file), &info) == -GRAVITY_EINVAL);
    make_valid(file, sizeof(file));
    put64(file + 24, UINT64_MAX);
    assert(gravity_app_validate(file, sizeof(file), &info) == -GRAVITY_EINVAL);
    make_valid(file, sizeof(file));
    put64(file + 32, 3);
    assert(gravity_app_validate(file, sizeof(file), &info) == -GRAVITY_EINVAL);
    make_valid(file, sizeof(file));
    file[40] = 1;
    assert(gravity_app_validate(file, sizeof(file), &info) == -GRAVITY_EINVAL);

    puts("Gravity GAPP format validation: PASS.");
    return 0;
}
