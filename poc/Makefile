RUST_MODULES = 	\
	cube			\
	console		\
	overlay

ZIG_MODULES =	\
	time_manager

C_MODULES =	\
	reset_cube

GO_MODULES =	\
  bg_manager

MODULES = $(RUST_MODULES) $(ZIG_MODULES) $(C_MODULES) $(GO_MODULES)

all: run

init:
	@mkdir -p modules

$(RUST_MODULES): init
	@echo -e "\x1b[1;33m[COMPILING]\x1b[0;90m $@ (Rust) in WASM...\x1b[0m"
	@cargo component build -p $@ --release && \
	cp target/wasm32-wasip1/release/$@.wasm modules/$@.wasm && \
	echo -e "\x1b[1;32m[OK]\x1b[94;3m $@ \x1b[0;90mcompiled & copied to \x1b[4;90mmodules/\x1b[0;90m !\x1b[0m" || \
	echo -e "\x1b[1;31m[ERROR]\x1b[0;90m Failed to compile \x1b[31;3m$@\x1b[0m"

$(ZIG_MODULES): init
	@WORLD_NAME=$$(cat libs/$@/.world 2>/dev/null); \
	if [ -z "$$WORLD_NAME" ]; then \
		echo -e "\x1b[1;31m[ERROR]\x1b[0;90m File .world is missing in \x1b[31;3m/libs/$@\x1b[0m"; \
	else \
		echo -e "\x1b[1;33m[COMPILING]\x1b[0;90m $@ (Zig) in WASM...\x1b[0m"; \
		WORLD_UNDERSCORE=$$(echo "$$WORLD_NAME" | tr '-' '_'); \
		cd libs/$@ && \
		rm -rf .zig-cache && \
		wit-bindgen c ../../wit --world $$WORLD_NAME && \
		echo "typedef unsigned long size_t; void free(void *ptr); void *realloc(void *ptr, size_t size); void abort(void);" > stdlib.h && \
		echo "typedef unsigned long size_t; size_t strlen(const char *s); void *memcpy(void *dest, const void *src, size_t n);" > string.h && \
		zig build-obj -target wasm32-freestanding -O ReleaseFast -fcompiler-rt -I. main.zig && \
		zig cc -target wasm32-freestanding -O3 -I. -c $$WORLD_UNDERSCORE.c -o $$WORLD_UNDERSCORE.o && \
		wasm-ld --no-entry main.o $$WORLD_UNDERSCORE.o $$WORLD_UNDERSCORE\_component_type.o -o core.wasm && \
		wasm-tools component new core.wasm -o ../../modules/$@.wasm && \
		rm -rf .zig-cache core.wasm main.o $$WORLD_UNDERSCORE.o $$WORLD_UNDERSCORE.h $$WORLD_UNDERSCORE.c $$WORLD_UNDERSCORE\_component_type.o stdlib.h string.h && \
		echo -e "\x1b[1;32m[OK]\x1b[94;3m $@ \x1b[0;90mcompiled & copied to \x1b[4;90mmodules/\x1b[0;90m !\x1b[0m" || \
		echo -e "\x1b[1;31m[ERROR]\x1b[0;90m Failed to compile \x1b[31;3m$@\x1b[0m"; \
	fi

$(C_MODULES): init
	@WORLD_NAME=$$(cat libs/$@/.world 2>/dev/null); \
	if [ -z "$$WORLD_NAME" ]; then \
		echo -e "\x1b[1;31m[ERROR]\x1b[0;90m File .world is missing in \x1b[31;3m/libs/$@\x1b[0m"; \
	else \
		echo -e "\x1b[1;33m[COMPILING]\x1b[0;90m $@ (C via Zig) in WASM...\x1b[0m"; \
		WORLD_UNDERSCORE=$$(echo "$$WORLD_NAME" | tr '-' '_'); \
		cd libs/$@ && \
		rm -rf .zig-cache && \
		wit-bindgen c ../../wit --world $$WORLD_NAME && \
		echo "typedef unsigned long size_t; void free(void *ptr); void *realloc(void *ptr, size_t size); void abort(void);" > stdlib.h && \
		echo "typedef unsigned long size_t; size_t strlen(const char *s); void *memcpy(void *dest, const void *src, size_t n);" > string.h && \
		zig cc -target wasm32-freestanding -O3 -I. -c main.c -o main.o && \
		zig cc -target wasm32-freestanding -O3 -I. -c $$WORLD_UNDERSCORE.c -o $$WORLD_UNDERSCORE.o && \
		wasm-ld --no-entry --allow-undefined main.o $$WORLD_UNDERSCORE.o $$WORLD_UNDERSCORE\_component_type.o -o core.wasm && \
		wasm-tools component new core.wasm -o ../../modules/$@.wasm && \
		rm -rf .zig-cache core.wasm $$WORLD_UNDERSCORE.h $$WORLD_UNDERSCORE.c $$WORLD_UNDERSCORE\_component_type.o main.o $$WORLD_UNDERSCORE.o stdlib.h string.h && \
		echo -e "\x1b[1;32m[OK]\x1b[94;3m $@ \x1b[0;90mcompiled & copied to \x1b[4;90mmodules/\x1b[0;90m !\x1b[0m" || \
		echo -e "\x1b[1;31m[ERROR]\x1b[0;90m Failed to compile \x1b[31;3m$@\x1b[0m"; \
	fi

$(GO_MODULES): init
	@WORLD_NAME=$$(cat libs/$@/.world 2>/dev/null); \
	if [ -z "$$WORLD_NAME" ]; then \
		echo -e "\x1b[1;31m[ERROR]\x1b[0;90m File .world is missing in \x1b[31;3m/libs/$@\x1b[0m";	\
	else \
		echo -e "\x1b[1;33m[COMPILING]\x1b[0;90m $@ (Go) in WASM...\x1b[0m"; \
		cd libs/$@ && \
		wit-bindgen go ../../wit --world $$WORLD_NAME && \
		if [ ! -f wasi_snapshot_preview1.reactor.wasm ]; then curl -sOL https://github.com/bytecodealliance/wasmtime/releases/download/v39.0.1/wasi_snapshot_preview1.reactor.wasm; fi && \
		go mod tidy && \
		GOARCH=wasm GOOS=wasip1 go build -o core.wasm -buildmode=c-shared -ldflags=-checklinkname=0 && \
		wasm-tools component embed ../../wit --world $$WORLD_NAME core.wasm -o core-with-wit.wasm && \
		wasm-tools component new --adapt wasi_snapshot_preview1.reactor.wasm core-with-wit.wasm -o ../../modules/$@.wasm && \
		rm -f core.wasm core-with-wit.wasm && \
		echo -e "\x1b[1;32m[OK]\x1b[94;3m $@ \x1b[0;90mcompiled & copied to \x1b[4;90mmodules/\x1b[0;90m !\x1b[0m" || \
		echo -e "\x1b[1;31m[ERROR]\x1b[0;90m Failed to compile \x1b[31;3m$@\x1b[0m"; \
	fi


build-modules: $(MODULES)

run: build-modules
	@echo -e "\x1b[1;36m[RUN]\x1b[0;90m Core\x1b[0m"
	@cargo run -p core --release

clean:
	@echo -e "\x1b[1;35m[CLEAN]\x1b[0;90m Cargo & \x1b[4;90mmodules/\x1b[0m"
	@cargo clean
	@rm -rf modules/
	@find libs/ -name "wasi_snapshot_preview1.reactor.wasm" -delete
	@find libs/ -name ".zig-cache" -exec rm -rf {} + 2>/dev/null || true
	@find libs/ -name "zig-out" -exec rm -rf {} + 2>/dev/null || true

re: clean all

.PHONY: all init $(MODULES) build-modules run clean re
