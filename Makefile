.PHONY: all core napi uniffi python kotlin swift check clean test

all: check napi python kotlin swift

check:
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo check -p open-mtr-core

core:
	cargo check -p open-mtr-core

napi:
	cd napi && npm ci && npm run build

uniffi:
	cargo build -p open-mtr-uniffi --release

python: uniffi
	uniffi-bindgen generate uniffi/src/mtr_sinir.udl --language python --out-dir bindings/python/
	cd bindings/python && pip install -e .

kotlin: uniffi
	uniffi-bindgen generate uniffi/src/mtr_sinir.udl --language kotlin --out-dir bindings/kotlin/

swift: uniffi
	uniffi-bindgen generate uniffi/src/mtr_sinir.udl --language swift --out-dir bindings/swift/

test:
	cargo test

clean:
	cargo clean
	rm -rf bindings/ napi/*.node napi/index.js napi/index.d.ts
