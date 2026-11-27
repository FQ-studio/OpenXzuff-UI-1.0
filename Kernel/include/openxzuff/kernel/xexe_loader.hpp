#pragma once
#include <gravity/abi.h>
#include <cstddef>
#include <cstdint>
#include <span>

namespace openxzuff::kernel {

inline constexpr std::size_t xexe_header_size = GRAVITY_XEXE_HEADER_SIZE;
inline constexpr std::uint16_t xexe_format_version = GRAVITY_XEXE_FORMAT_VERSION;
inline constexpr std::uint16_t xexe_architecture_rv64 = GRAVITY_XEXE_ARCH_RV64;
inline constexpr std::uint16_t xexe_gravity_abi_v1 = GRAVITY_ABI_VERSION;

enum class XexeValidationError : std::uint8_t {
    None, TooSmall, BadMagic, UnsupportedFormat, BadHeaderSize,
    WrongArchitecture, WrongAbi, ReservedFlagsSet, EmptyCode,
    SizeMismatch, EntryOutsideCode
};

struct XexeView {
    std::span<const std::uint8_t> code;
    std::uint64_t entry_offset;
};

// Validates an in-memory .xexe image only. It does not map memory or start a process.
XexeValidationError validate_xexe(std::span<const std::uint8_t> file, XexeView& view) noexcept;
const char* xexe_validation_error_name(XexeValidationError error) noexcept;

} // namespace openxzuff::kernel
