.PHONY: build test fmt fmt-check lint clean deploy-testnet help

## build: Compile contracts to WASM (release)
build:
	stellar contract build

## test: Run all unit tests
test:
	cargo test

## fmt: Auto-format all source files
fmt:
	cargo fmt

## fmt-check: Verify formatting without modifying files (used in CI)
fmt-check:
	cargo fmt --check

## lint: Run Clippy and fail on any warning
lint:
	cargo clippy --all-targets -- -D warnings

## clean: Remove build artefacts
clean:
	cargo clean

## deploy-testnet: Build and deploy both contracts to Stellar Testnet
deploy-testnet:
	NETWORK=testnet ./scripts/deploy.sh

## help: List all available targets
help:
	@grep -E '^## ' Makefile | sed 's/^## //' | column -t -s ':'
