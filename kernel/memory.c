#include <stddef.h>

/* Minimal freestanding C runtime primitive required by compiler-generated code.
 * Volatile writes prevent the compiler from lowering this loop back to memset. */
void *memset(void *destination, int value, size_t count)
{
    volatile unsigned char *out = (volatile unsigned char *)destination;
    while (count != 0u) {
        *out++ = (unsigned char)value;
        --count;
    }
    return destination;
}
