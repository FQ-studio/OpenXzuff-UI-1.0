#ifndef GRAVITY_ABI_H
#define GRAVITY_ABI_H

#include <stdint.h>

#define GRAVITY_ABI_VERSION 1u
#define GRAVITY_XEXE_FORMAT_VERSION 1u
#define GRAVITY_XEXE_HEADER_SIZE 32u
#define GRAVITY_XEXE_ARCH_RV64 1u

#define GRAVITY_SYS_EXIT 0u
#define GRAVITY_SYS_SEND_MESSAGE 1u
#define GRAVITY_SYS_RECEIVE_MESSAGE 2u
#define GRAVITY_SYS_MAP_MEMORY 3u
#define GRAVITY_SYS_YIELD 4u

/* Gravity ABI returns errors as negative errno values in a0. Payloads whose
 * signed value is -1..-4095 are reserved to avoid colliding with errors. */
#define GRAVITY_EAGAIN 11
#define GRAVITY_EFAULT 14
#define GRAVITY_EINVAL 22
#define GRAVITY_ENOSYS 38

#endif /* GRAVITY_ABI_H */
