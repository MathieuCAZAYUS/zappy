PLUGINS = 	\
	console

all: run

init:
	@mkdir -p plugins

build-plugins: init
	@for plugin in $(PLUGINS); do \
		echo -e "\x1b[1;33m[COMPILING]\x1b[0;90m $$plugin in WASM..."; \
		cargo component build -p $$plugin --release; \
		cp target/wasm32-wasip1/release/$$plugin.wasm plugins/$$plugin.wasm; \
		echo -e "\x1b[1;32m[OK]\x1b[0;90m $$plugin compiled & copied to \x1b[4;90mplugins/\x1b[0;90m !\x1b[0m"; \
	done

run: build-plugins
	@echo -e "\x1b[1;36m[RUN]\x1b[0;90m Core\x1b[0m"
	@cargo run -p core --release

clean:
	@echo -e "\x1b[1;35m[CLEAN]\x1b[0;90m Cargo & \x1b[4;90mplugins/\x1b[0m"
	@cargo clean
	@rm -rf plugins/

re: clean all

.PHONY: all init build-plugins run clean re
