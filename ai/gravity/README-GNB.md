# Gravity Native Binary (GNB) v1

Gravity AI uses GNB as its private native-code deployment container.

Pipeline:
1. GAIL source (.gxk) is parsed by the GAIL compiler.
2. The compiler emits C++20 translation units.
3. Clang/LLVM targets RV64 and links a static ELF64 little-endian ET_EXEC image.
4. gravity-elf-to-gnb extracts PT_LOAD segments into a .gnb file.
5. The Gravity GNB loader validates the container, maps segments, zero-fills BSS, enforces permissions, and transfers control to the entry address.

## GNB v1 layout

All integers are unsigned little-endian. Header size: 56 bytes.

| Offset | Size | Meaning |
|---:|---:|---|
| 0 | 4 | Magic: GNB1 |
| 4 | 2 | Version: 1 |
| 6 | 2 | Header size: 56 |
| 8 | 4 | Architecture: 1 = RV64 |
| 12 | 4 | Flags: zero in v1 |
| 16 | 8 | RV64 entry virtual address |
| 24 | 4 | Segment count: 1–64 |
| 28 | 4 | Reserved |
| 32 | 8 | Segment table offset: 56 |
| 40 | 8 | Payload offset |
| 48 | 8 | FNV-1a 64-bit checksum over bytes [56, EOF) |

Each 48-byte segment record contains five u64 fields (virtual address, memory size, file size, payload offset, alignment), then ELF flags u32 and reserved u32. Segment payloads follow the table. memory_size beyond file_size is zero-filled BSS.

## Constraints and security

- Accepts ELF64 little-endian RISC-V (EM_RISCV=243), static ET_EXEC only, with 1–64 valid PT_LOAD segments.
- Rejects ET_REL objects, dynamic images, non-RISC-V architecture and invalid segment bounds.
- GNB is not ELF and cannot be run by a generic ELF loader. OpenXzuff needs a GNB loader to validate/map it.
- The checksum detects accidental corruption; FNV-1a is not a cryptographic signature.
- GNB contains native RV64 machine code, not bytecode and not a new instruction set.
