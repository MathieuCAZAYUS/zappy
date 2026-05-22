MODULES = 	\
	cube			\
	console		\
	overlay

all: run

init:
	@mkdir -p modules

build-modules: init
	@for module in $(MODULES); do \
		echo -e "\x1b[1;33m[COMPILING]\x1b[0;90m $$module in WASM..."; \
		cargo component build -p $$module --release; \
		cp target/wasm32-wasip1/release/$$module.wasm modules/$$module.wasm; \
		echo -e "\x1b[1;32m[OK]\x1b[94;3m $$module \x1b[0;90mcompiled & copied to \x1b[4;90mmodules/\x1b[0;90m !\x1b[0m"; \
	done

run: build-modules
	@echo -e "\x1b[1;36m[RUN]\x1b[0;90m Core\x1b[0m"
	@cargo run -p core --release

clean:
	@echo -e "\x1b[1;35m[CLEAN]\x1b[0;90m Cargo & \x1b[4;90mmodules/\x1b[0m"
	@cargo clean
	@rm -rf modules/

re: clean all

.PHONY: all init build-modules run clean re
