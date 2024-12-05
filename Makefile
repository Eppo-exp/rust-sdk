SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

WASM_TARGET=wasm32-wasi
FASTLY_PACKAGE=fastly-edge-assignments
BUILD_DIR=target/$(WASM_TARGET)/release
WASM_FILE=$(BUILD_DIR)/$(FASTLY_PACKAGE).wasm

# Help target for easy documentation
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all                           - Default target (build workspace)"
	@echo "  workspace-build               - Build the entire workspace excluding the Fastly package"
	@echo "  workspace-test                - Test the entire workspace excluding the Fastly package"
	@echo "  fastly-edge-assignments-build - Build only the Fastly package for WASM"
	@echo "  fastly-edge-assignments-test  - Test only the Fastly package"
	@echo "  clean                         - Clean all build artifacts"

.PHONY: test
test: ${testDataDir}
	npm test

# Build the entire workspace excluding the `fastly-edge-assignments` package
.PHONY: workspace-build
workspace-build:
	cargo build

# Run tests for the entire workspace excluding the `fastly-edge-assignments` package
.PHONY: workspace-test
workspace-test:
	cargo test

# Build only the `fastly-edge-assignments` package for WASM
.PHONY: fastly-edge-assignments-build
fastly-edge-assignments-build:
	rustup target add $(WASM_TARGET)
	cd fastly-edge-assignments
	cargo build --release --target $(WASM_TARGET)

# Test only the `fastly-edge-assignments` package
.PHONY: fastly-edge-assignments-test
fastly-edge-assignments-test:
	cd fastly-edge-assignments
	cargo test --target $(WASM_TARGET)
