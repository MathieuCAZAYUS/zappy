RUST_MODULES = 	\
	console		\
	crosshair	\
	game_view	\
	obj_viewer \
	overlay	\
	zappy_net

MODULES = $(RUST_MODULES)

all: gui

init:
	@mkdir -p gui/modules

$(RUST_MODULES): init
	@echo -e "\x1b[1;33m[COMPILING]\x1b[0;90m $@ (Rust) in WASM...\x1b[0m"
	@cd gui/ && \
	cargo component build -p $@ --release && \
	cp target/wasm32-wasip1/release/$@.wasm modules/$@.wasm && \
	echo -e "\x1b[1;32m[OK]\x1b[94;3m $@ \x1b[0;90mcompiled & copied to \x1b[4;90mmodules/\x1b[0;90m !\x1b[0m" || \
	echo -e "\x1b[1;31m[ERROR]\x1b[0;90m Failed to compile \x1b[31;3m$@\x1b[0m"

build-modules: $(MODULES)

gui: build-modules
	@echo -e "\x1b[1;36m[RUN]\x1b[0;90m GUI Core\x1b[0m"
	@cd gui/ && \
	cargo run -p core --release

clean-gui:
	@echo -e "\x1b[1;35m[CLEAN]\x1b[0;90m GUI Cargo & \x1b[4;90mmodules/\x1b[0m"
	@cd gui/ && \
	cargo clean && \
	rm -rf gui/modules/

fclean: clean-gui

re: fclean all

.PHONY: all init $(MODULES) build-modules gui clean-gui fclean re
