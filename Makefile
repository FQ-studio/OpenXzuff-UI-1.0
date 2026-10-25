# Cross-build foundations. Requires Lua, Rust/Cargo and a bare-metal RV64 toolchain.
RISCV_PREFIX ?= riscv64-unknown-elf-

.PHONY: help check-all check-lua check-rust check-ai-cpp check-boot-alloc check-memory-map check-fdt check-app check-installer check-app-hub check-gravity-rust clean kernel-rv64
help:
	@echo 'check-all runs all host-side checks and the RV64 cross-build.'
	@echo 'Targets: check-lua, check-rust, check-gravity-rust, check-app, check-installer, check-app-hub, kernel-rv64, clean'
	@echo 'check-gravity-rust tests the Rust Gravity Kernel modules on the host.'
		@echo 'kernel-rv64 builds an RV64 early payload; it is not a complete OS.'

check-all: check-lua check-rust check-ai-cpp check-boot-alloc check-memory-map check-fdt check-app check-installer check-app-hub kernel-rv64

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

check-gravity-rust:
	cargo test --manifest-path kernel/Cargo.toml

check-boot-alloc check-memory-map check-fdt: check-gravity-rust

check-app: check-gravity-rust

kernel-rv64: build/openxzuff-rv64.elf

build/openxzuff-rv64.elf: kernel/Cargo.toml kernel/.cargo/config.toml kernel/src/main.rs kernel/src/memory.rs kernel/src/fdt.rs kernel/src/syscall.rs kernel/src/app.rs kernel/arch/riscv/entry.S kernel/arch/riscv/trap.S kernel/linker-riscv.ld
	mkdir -p build
	rustup target add riscv64gc-unknown-none-elf
	cargo build --manifest-path kernel/Cargo.toml --release --target riscv64gc-unknown-none-elf
	cp kernel/target/riscv64gc-unknown-none-elf/release/gravity-kernel $@

clean:
	rm -rf build
