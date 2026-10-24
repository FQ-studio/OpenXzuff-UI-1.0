# Cross-build foundations. Requires Lua, Rust/Cargo and a bare-metal RV64 toolchain.
RISCV_PREFIX ?= riscv64-unknown-elf-
RV_CFLAGS := -march=rv64gc -mabi=lp64d -mcmodel=medany -ffreestanding -fno-stack-protector -fno-pic -O2 -Wall -Wextra -Ikernel/include
RV_ASFLAGS := $(RV_CFLAGS)
RV_LDFLAGS := -nostdlib -Wl,-T,kernel/linker-riscv.ld -Wl,--build-id=none

.PHONY: help check-lua check-rust check-ai-cpp check-boot-alloc check-memory-map check-fdt check-app check-installer check-app-hub clean kernel-rv64
help:
	@echo 'Targets: check-lua, check-rust, check-ai-cpp, check-boot-alloc, check-memory-map, check-fdt, check-app, check-installer, check-app-hub, kernel-rv64, clean'
	@echo 'check-boot-alloc tests the bounded early-boot allocator on the host.'
	@echo 'check-memory-map tests RAM range tracking and reservation logic.'
	@echo 'check-fdt tests parsing a bootloader-provided Flattened Device Tree.'
	@echo 'kernel-rv64 builds an RV64 early payload; it is not a complete OS.'

check-app-hub:
	cargo test --manifest-path services/app-hub/Cargo.toml

check-installer:
	mkdir -p build
	$(CXX) -std=c++17 -Wall -Wextra -Werror tools/oxz-install.cpp -o build/oxz-install
	$(CXX) -std=c++17 -Wall -Wextra -Werror tests/oxz_install_cpp_test.cpp -o build/oxz-install-test
	./build/oxz-install-test

check-lua:
	lua tests/ui_spec.lua
	lua tests/package_installer_spec.lua
	lua tests/xzuff_app_hub_spec.lua
	find src tests runtime/lua -name '*.lua' -print0 | xargs -0 -n1 luac -p

check-rust:
	cargo test --manifest-path services/sql-ai/Cargo.toml

check-ai-cpp:
	$(CXX) -std=c++17 -Wall -Wextra -Werror -Iai/runtime/include -fsyntax-only ai/runtime/src/ai_runtime.cpp

check-boot-alloc:
	mkdir -p build
	$(CC) -std=c11 -Wall -Wextra -Werror -Ikernel/include kernel/boot_alloc.c tests/boot_alloc_test.c -o build/boot_alloc_test
	./build/boot_alloc_test

check-memory-map:
	mkdir -p build
	$(CC) -std=c11 -Wall -Wextra -Werror -Ikernel/include kernel/memory_map.c tests/memory_map_test.c -o build/memory_map_test
	./build/memory_map_test

check-fdt:
	mkdir -p build
	$(CC) -std=c11 -Wall -Wextra -Werror -Ikernel/include kernel/fdt.c kernel/memory_map.c tests/fdt_memory_test.c -o build/fdt_memory_test
	./build/fdt_memory_test

check-app:
	mkdir -p build
	$(CC) -std=c11 -Wall -Wextra -Werror -Ikernel/include kernel/app.c tests/app_format_test.c -o build/app_format_test
	./build/app_format_test

kernel-rv64: build/openxzuff-rv64.elf

build/openxzuff-rv64.elf: kernel/kernel.c kernel/app.c kernel/syscall.c kernel/trap.c kernel/boot_alloc.c kernel/memory_map.c kernel/fdt.c kernel/memory.c kernel/include/gravity/app.h kernel/include/gravity/fdt.h kernel/include/gravity/memory_map.h kernel/include/gravity/syscall.h kernel/include/gravity/boot_alloc.h kernel/arch/riscv/entry.S kernel/arch/riscv/trap.S kernel/linker-riscv.ld
	mkdir -p build
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/kernel.c -o build/kernel.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/app.c -o build/app.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/boot_alloc.c -o build/boot_alloc.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/memory_map.c -o build/memory_map.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -fno-builtin-memset -c kernel/fdt.c -o build/fdt.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -fno-builtin-memset -c kernel/memory.c -o build/memory.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/syscall.c -o build/syscall.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/trap.c -o build/trap.o
	$(RISCV_PREFIX)gcc $(RV_ASFLAGS) -c kernel/arch/riscv/entry.S -o build/entry.o
	$(RISCV_PREFIX)gcc $(RV_ASFLAGS) -c kernel/arch/riscv/trap.S -o build/trap_entry.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) $(RV_LDFLAGS) build/entry.o build/trap_entry.o build/kernel.o build/app.o build/boot_alloc.o build/memory_map.o build/fdt.o build/memory.o build/trap.o build/syscall.o -o $@

clean:
	rm -rf build
