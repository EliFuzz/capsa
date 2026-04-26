CRATE_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
REPO_ROOT := $(abspath $(CRATE_DIR)/../../..)
VENDOR_DIR := $(REPO_ROOT)/vendor/vectorscan
BUILD_DIR := $(CRATE_DIR)/build
CMAKE_DIR := $(BUILD_DIR)/cmake
INSTALL_DIR := $(BUILD_DIR)/vectorscan

CMAKE ?= cmake
CARGO ?= cargo
CMAKE_BUILD_TYPE ?= Release

export VECTORSCAN_BUILD_DIR := $(INSTALL_DIR)
export VECTORSCAN_INCLUDE_DIR := $(INSTALL_DIR)/include

CMAKE_FLAGS := \
	-DCMAKE_BUILD_TYPE=$(CMAKE_BUILD_TYPE) \
	-DCMAKE_INSTALL_PREFIX=$(INSTALL_DIR) \
	-DBUILD_SHARED_LIBS=OFF \
	-DBUILD_STATIC_LIBS=ON \
	-DFAT_RUNTIME=OFF \
	-DWARNINGS_AS_ERRORS=OFF \
	-DBUILD_EXAMPLES=OFF \
	-DBUILD_BENCHMARKS=OFF \
	-DBUILD_DOC=OFF \
	-DBUILD_TOOLS=OFF \
	-DBUILD_UNIT=OFF

CARGO_ARGS := -p core-regex --manifest-path $(REPO_ROOT)/Cargo.toml

.PHONY: build sync vectorscan bindings test clippy clean

build: vectorscan
	@$(CARGO) build $(CARGO_ARGS)

sync:
	@git -C $(REPO_ROOT) submodule update --init --remote --recursive --merge vendor/vectorscan

vectorscan: sync
	@$(CMAKE) -S $(VENDOR_DIR) -B $(CMAKE_DIR) $(CMAKE_FLAGS)
	@$(CMAKE) --build $(CMAKE_DIR) --config $(CMAKE_BUILD_TYPE) --parallel
	@$(CMAKE) --install $(CMAKE_DIR) --config $(CMAKE_BUILD_TYPE)

bindings: vectorscan
	@$(CARGO) build $(CARGO_ARGS) --features gen

test: vectorscan
	@$(CARGO) test $(CARGO_ARGS)

clippy: vectorscan
	@$(CARGO) clippy $(CARGO_ARGS) --all-targets -- -D warnings

clean:
	@$(CMAKE) -E rm -rf $(BUILD_DIR)
