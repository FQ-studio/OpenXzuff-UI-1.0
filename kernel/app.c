#include "gravity/app.h"
#include "gravity/syscall.h"

static uint16_t read_le16(const uint8_t *p)
{
    return (uint16_t)((uint16_t)p[0] | ((uint16_t)p[1] << 8));
}

static uint32_t read_le32(const uint8_t *p)
{
    return (uint32_t)p[0] |
           ((uint32_t)p[1] << 8) |
           ((uint32_t)p[2] << 16) |
           ((uint32_t)p[3] << 24);
}

static uint64_t read_le64(const uint8_t *p)
{
    return (uint64_t)read_le32(p) | ((uint64_t)read_le32(p + 4) << 32);
}

int gravity_app_validate(const void *file, size_t file_size,
                         gravity_app_info_t *out_info)
{
    const uint8_t *p = (const uint8_t *)file;
    gravity_app_info_t info;

    if (p == NULL || out_info == NULL || file_size < GRAVITY_APP_HEADER_SIZE)
        return -GRAVITY_EINVAL;

    if (p[0] != 'G' || p[1] != 'A' || p[2] != 'P' || p[3] != 'P')
        return -GRAVITY_EINVAL;
    if (read_le16(p + 4) != GRAVITY_APP_VERSION ||
        read_le16(p + 6) != GRAVITY_APP_HEADER_SIZE)
        return -GRAVITY_EINVAL;

    info.architecture = read_le32(p + 8);
    info.flags = read_le32(p + 12);
    info.entry_offset = read_le64(p + 16);
    info.image_size = read_le64(p + 24);
    info.memory_size = read_le64(p + 32);

    /* Reserved bytes must be zero so future extensions fail closed. */
    if (read_le64(p + 40) != 0)
        return -GRAVITY_EINVAL;
    if (info.architecture != GRAVITY_APP_ARCH_RV64 ||
        info.flags != GRAVITY_APP_FLAG_NONE)
        return -GRAVITY_EINVAL;
    if (info.image_size == 0 || info.image_size > (uint64_t)(file_size - GRAVITY_APP_HEADER_SIZE))
        return -GRAVITY_EINVAL;
    if (info.entry_offset >= info.image_size || info.memory_size < info.image_size)
        return -GRAVITY_EINVAL;

    *out_info = info;
    return 0;
}
