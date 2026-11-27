#include "openxzuff/kernel/xexe_loader.hpp"

namespace openxzuff::kernel {
namespace {
constexpr char magic[] = GRAVITY_XEXE_MAGIC;
static_assert(sizeof(magic) == 8, "Gravity .xexe magic must remain exactly 8 bytes");

std::uint16_t read16(std::span<const std::uint8_t> data, std::size_t offset) noexcept {
    return static_cast<std::uint16_t>(data[offset]) |
           static_cast<std::uint16_t>(static_cast<std::uint16_t>(data[offset + 1]) << 8U);
}

std::uint32_t read32(std::span<const std::uint8_t> data, std::size_t offset) noexcept {
    std::uint32_t value = 0;
    for (std::size_t i = 0; i < 4; ++i) {
        value |= static_cast<std::uint32_t>(data[offset + i]) << (8U * i);
    }
    return value;
}

std::uint64_t read64(std::span<const std::uint8_t> data, std::size_t offset) noexcept {
    std::uint64_t value = 0;
    for (std::size_t i = 0; i < 8; ++i) {
        value |= static_cast<std::uint64_t>(data[offset + i]) << (8U * i);
    }
    return value;
}
}

XexeValidationError validate_xexe(std::span<const std::uint8_t> file, XexeView& view) noexcept {
    view = {};
    if (file.size() < xexe_header_size) return XexeValidationError::TooSmall;
    for (std::size_t i = 0; i < sizeof(magic); ++i) {
        if (file[i] != static_cast<std::uint8_t>(magic[i])) return XexeValidationError::BadMagic;
    }
    if (read16(file, GRAVITY_XEXE_OFFSET_FORMAT_VERSION) != xexe_format_version) {
        return XexeValidationError::UnsupportedFormat;
    }
    if (read16(file, GRAVITY_XEXE_OFFSET_HEADER_SIZE) != xexe_header_size) {
        return XexeValidationError::BadHeaderSize;
    }
    if (read16(file, GRAVITY_XEXE_OFFSET_ARCHITECTURE) != xexe_architecture_rv64) {
        return XexeValidationError::WrongArchitecture;
    }
    if (read16(file, GRAVITY_XEXE_OFFSET_ABI_VERSION) != xexe_gravity_abi_v1) {
        return XexeValidationError::WrongAbi;
    }
    if (read32(file, GRAVITY_XEXE_OFFSET_FLAGS) != GRAVITY_XEXE_RESERVED_FLAGS) {
        return XexeValidationError::ReservedFlagsSet;
    }

    const std::uint64_t entry = read64(file, GRAVITY_XEXE_OFFSET_ENTRY);
    const std::uint64_t codeSize = read32(file, GRAVITY_XEXE_OFFSET_CODE_SIZE);
    if (codeSize == 0) return XexeValidationError::EmptyCode;
    if (codeSize != static_cast<std::uint64_t>(file.size() - xexe_header_size)) {
        return XexeValidationError::SizeMismatch;
    }
    if (entry < xexe_header_size || entry >= xexe_header_size + codeSize) {
        return XexeValidationError::EntryOutsideCode;
    }
    view.code = file.subspan(xexe_header_size, static_cast<std::size_t>(codeSize));
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
