#include "openxzuff/kernel/xexe_loader.hpp"
#include <limits>

namespace openxzuff::kernel {
namespace {
constexpr std::uint8_t magic[8] = {'X', 'Z', 'X', 'E', 'X', 'E', '1', 0};
std::uint16_t read16(std::span<const std::uint8_t> data, std::size_t offset) noexcept {
    return static_cast<std::uint16_t>(data[offset]) |
           static_cast<std::uint16_t>(static_cast<std::uint16_t>(data[offset + 1]) << 8U);
}
std::uint32_t read32(std::span<const std::uint8_t> data, std::size_t offset) noexcept {
    std::uint32_t value = 0;
    for (std::size_t i = 0; i < 4; ++i) value |= static_cast<std::uint32_t>(data[offset + i]) << (8U * i);
    return value;
}
std::uint64_t read64(std::span<const std::uint8_t> data, std::size_t offset) noexcept {
    std::uint64_t value = 0;
    for (std::size_t i = 0; i < 8; ++i) value |= static_cast<std::uint64_t>(data[offset + i]) << (8U * i);
    return value;
}
}

XexeValidationError validate_xexe(std::span<const std::uint8_t> file, XexeView& view) noexcept {
    view = {};
    if (file.size() < xexe_header_size) return XexeValidationError::TooSmall;
    for (std::size_t i = 0; i < 8; ++i) if (file[i] != magic[i]) return XexeValidationError::BadMagic;
    if (read16(file, 8) != xexe_format_version) return XexeValidationError::UnsupportedFormat;
    if (read16(file, 10) != xexe_header_size) return XexeValidationError::BadHeaderSize;
    if (read16(file, 12) != xexe_architecture_rv64) return XexeValidationError::WrongArchitecture;
    if (read16(file, 14) != xexe_gravity_abi_v1) return XexeValidationError::WrongAbi;
    if (read32(file, 28) != 0) return XexeValidationError::ReservedFlagsSet;
    const std::uint64_t entry = read64(file, 16);
    const std::uint64_t code_size = read32(file, 24);
    if (code_size == 0) return XexeValidationError::EmptyCode;
    if (code_size > static_cast<std::uint64_t>(file.size() - xexe_header_size) ||
        code_size != static_cast<std::uint64_t>(file.size() - xexe_header_size)) {
        return XexeValidationError::SizeMismatch;
    }
    if (entry < xexe_header_size || entry >= xexe_header_size + code_size) return XexeValidationError::EntryOutsideCode;
    view.code = file.subspan(xexe_header_size, static_cast<std::size_t>(code_size));
    view.entry_offset = entry;
    return XexeValidationError::None;
}

const char* xexe_validation_error_name(XexeValidationError error) noexcept {
    switch (error) {
    case XexeValidationError::None: return "none";
    case XexeValidationError::TooSmall: return "file smaller than header";
    case XexeValidationError::BadMagic: return "bad .xexe magic";
    case XexeValidationError::UnsupportedFormat: return "unsupported format version";
    case XexeValidationError::BadHeaderSize: return "invalid header size";
    case XexeValidationError::WrongArchitecture: return "wrong architecture";
    case XexeValidationError::WrongAbi: return "wrong Gravity ABI version";
    case XexeValidationError::ReservedFlagsSet: return "reserved flags must be zero";
    case XexeValidationError::EmptyCode: return "empty code segment";
    case XexeValidationError::SizeMismatch: return "code size does not match file size";
    case XexeValidationError::EntryOutsideCode: return "entry offset outside code segment";
    }
    return "unknown validation error";
}

} // namespace openxzuff::kernel