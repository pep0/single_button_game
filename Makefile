.PHONY: help run editor build release wasm wasm-serve clean

help:           ## Show this help
	@awk 'BEGIN{FS=":.*##"} /^[a-zA-Z_-]+:.*##/{printf "  %-14s %s\n",$$1,$$2}' $(MAKEFILE_LIST)

run:            ## Run the game (dev build)
	cargo run --bin single_button_game

editor:         ## Run the level editor (dev build)
	cargo run --bin level_editor

build:          ## Dev build (no execution)
	cargo build

release:        ## Optimised native release build
	cargo build --release

wasm:           ## Build WASM bundle with Trunk (size-optimised)
	trunk build --release

wasm-serve:     ## Build WASM bundle and serve locally at http://localhost:8080
	trunk serve

clean:          ## Remove all build artefacts
	cargo clean
	rm -rf dist
