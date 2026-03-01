.PHONY: build-native build-wasm build-all test clean

build-native:
	cargo build --release

build-wasm:
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --out-dir dist/wasm --web target/wasm32-unknown-unknown/release/spacegame_bmad.wasm || true

build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

build-windows:
	cargo build --release --target x86_64-pc-windows-gnu || echo "Cross-compile requires mingw"

build-all: build-native build-wasm

test:
	cargo test

clean:
	cargo clean
	rm -rf dist/
