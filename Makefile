.PHONY: build test fmt fmt-check lint clean
build:
	stellar contract build
test:
	cargo test
fmt:
	cargo fmt
fmt-check:
	cargo fmt --check
lint:
	cargo clippy --all-targets -- -D warnings
clean:
	cargo clean
