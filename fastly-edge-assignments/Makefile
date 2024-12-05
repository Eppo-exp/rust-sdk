SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

WASM_TARGET=wasm32-wasi
BUILD_DIR=target/$(WASM_TARGET)/release
WASM_FILE=$(BUILD_DIR)/$(FASTLY_PACKAGE).wasm

# Help target for easy documentation
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build                         - Build the WASM target"
	@echo "  test                          - Run unit and integration tests"
	@echo "  clean                         - Clean all build artifacts"

.PHONY: clean
clean:
	rm -rf bin pkg

.PHONY: build
build:
	rustup target add $(WASM_TARGET)
	fastly compute build

.PHONY: test
test:
	cargo nextest run
