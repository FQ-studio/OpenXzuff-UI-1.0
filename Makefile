# Cross-build foundations. Requires Lua, Rust/Cargo and a bare-metal RV64 toolchain.
RISCV_PREFIX ?= riscv64-unknown-elf-
RV_CFLAGS := -march=rv64gc -mabi=lp64d -mcmodel=medany -ffreestanding -fno-stack-protector -fno-pic -O2 -Wall -Wextra -Ikernel/include
RV_ASFLAGS := $(RV_CFLAGS)
RV_LDFLAGS := -nostdlib -Wl,-T,kernel/linker-riscv.ld -Wl,--build-id=none

.PHONY: help check-lua check-rust check-ai-cpp clean kernel-rv64
help:
	@echo 'Targets: check-lua, check-rust, check-ai-cpp, kernel-rv64, clean'
	@echo 'kernel-rv64 builds only an early RV64 payload, not a complete bootable OS.'

check-lua:
	lua tests/ui_spec.lua
	find src tests runtime/lua -name '*.lua' -print0 | xargs -0 -n1 luac -p

check-rust:
	cargo test --manifest-path services/sql-ai/Cargo.toml

check-ai-cpp:
	$(CXX) -std=c++17 -Wall -Wextra -Werror -Iai/runtime/include -fsyntax-only ai/runtime/src/ai_runtime.cpp

kernel-rv64: build/openxzuff-rv64.elf

build/openxzuff-rv64.elf: kernel/kernel.c kernel/syscall.c kernel/include/gravity/syscall.h kernel/arch/riscv/entry.S kernel/linker-riscv.ld
	mkdir -p build
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/kernel.c -o build/kernel.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/syscall.c -o build/syscall.o
	$(RISCV_PREFIX)gcc $(RV_ASFLAGS) -c kernel/arch/riscv/entry.S -o build/entry.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) $(RV_LDFLAGS) build/entry.o build/kernel.o build/syscall.o -o $@

clean:
	rm -rf build
