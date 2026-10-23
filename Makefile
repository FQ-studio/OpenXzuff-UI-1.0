# Cross-build foundations. Requires Lua, Rust/Cargo and a bare-metal RV64 toolchain.
RISCV_PREFIX ?= riscv64-unknown-elf-
RV_CFLAGS := -march=rv64gc -mabi=lp64d -mcmodel=medany -ffreestanding -fno-stack-protector -fno-pic -O2 -Wall -Wextra -Ikernel/include
RV_ASFLAGS := $(RV_CFLAGS)
RV_LDFLAGS := -nostdlib -Wl,-T,kernel/linker-riscv.ld -Wl,--build-id=none

.PHONY: help check-lua check-rust check-ai-cpp check-boot-alloc clean kernel-rv64
help:
	@echo 'Targets: check-lua, check-rust, check-ai-cpp, check-boot-alloc, kernel-rv64, clean'
	@echo 'check-boot-alloc tests the bounded early-boot allocator on the host.'
	@echo 'kernel-rv64 builds an RV64 early payload with a supervisor trap/syscall-path smoke test; it is not a complete OS.'

check-lua:
	lua tests/ui_spec.lua
	find src tests runtime/lua -name '*.lua' -print0 | xargs -0 -n1 luac -p

check-rust:
	cargo test --manifest-path services/sql-ai/Cargo.toml

check-ai-cpp:
	$(CXX) -std=c++17 -Wall -Wextra -Werror -Iai/runtime/include -fsyntax-only ai/runtime/src/ai_runtime.cpp

check-boot-alloc:
	mkdir -p build
	$(CC) -std=c11 -Wall -Wextra -Werror -Ikernel/include kernel/boot_alloc.c tests/boot_alloc_test.c -o build/boot_alloc_test
	./build/boot_alloc_test

kernel-rv64: build/openxzuff-rv64.elf

build/openxzuff-rv64.elf: kernel/kernel.c kernel/syscall.c kernel/trap.c kernel/boot_alloc.c kernel/include/gravity/syscall.h kernel/include/gravity/boot_alloc.h kernel/arch/riscv/entry.S kernel/arch/riscv/trap.S kernel/linker-riscv.ld
	mkdir -p build
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/kernel.c -o build/kernel.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/boot_alloc.c -o build/boot_alloc.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/syscall.c -o build/syscall.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/trap.c -o build/trap.o
	$(RISCV_PREFIX)gcc $(RV_ASFLAGS) -c kernel/arch/riscv/entry.S -o build/entry.o
	$(RISCV_PREFIX)gcc $(RV_ASFLAGS) -c kernel/arch/riscv/trap.S -o build/trap_entry.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) $(RV_LDFLAGS) build/entry.o build/trap_entry.o build/kernel.o build/boot_alloc.o build/trap.o build/syscall.o -o $@

clean:
	rm -rf build
