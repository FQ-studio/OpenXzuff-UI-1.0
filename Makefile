# Cross-build foundations; requires a matching bare-metal toolchain.
RISCV_PREFIX ?= riscv64-unknown-elf-
RV_CFLAGS := -march=rv64gc -mabi=lp64d -mcmodel=medany -ffreestanding -fno-stack-protector -fno-pic -O2 -Wall -Wextra
RV_ASFLAGS := $(RV_CFLAGS)
RV_LDFLAGS := -nostdlib -Wl,-T,kernel/linker-riscv.ld -Wl,--build-id=none

.PHONY: help check-lua clean kernel-rv64
help:
	@echo 'Targets: check-lua, kernel-rv64, clean'
	@echo 'kernel-rv64 builds a minimal SBI-style RV64 payload; it is not yet a desktop OS.'

check-lua:
	lua tests/ui_spec.lua
	find src tests -name '*.lua' -print0 | xargs -0 -n1 luac -p

kernel-rv64: build/openxzuff-rv64.elf

build/openxzuff-rv64.elf: kernel/kernel.c kernel/arch/riscv/entry.S kernel/linker-riscv.ld
	mkdir -p build
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) -c kernel/kernel.c -o build/kernel.o
	$(RISCV_PREFIX)gcc $(RV_ASFLAGS) -c kernel/arch/riscv/entry.S -o build/entry.o
	$(RISCV_PREFIX)gcc $(RV_CFLAGS) $(RV_LDFLAGS) build/entry.o build/kernel.o -o $@

clean:
	rm -rf build
