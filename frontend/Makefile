# ENC Chart Viewer — build helper
#
# Prerequisites:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-pack
#   npm (nodejs >= 18)
#   (for Python styling) pip install maturin
#
# Quick start:
#   make wasm          # compile Rust → WASM, output → src/wasm-pkg/
#   make dev           # run Vite dev server (builds WASM first if needed)
#   make build         # production bundle → dist/

# Locate the two sibling repositories
BOAT_DIR   := $(realpath $(CURDIR)/../..)
WASM_CRATE := $(BOAT_DIR)/openenc-styling-rust/crates/s57style-wasm
WASM_OUT   := $(CURDIR)/src/wasm-pkg

# Python maturin target
PY_CRATE   := $(BOAT_DIR)/openenc-styling-rust/crates/s57style-python

.PHONY: all wasm wasm-release py-dev py-build deps dev build clean

all: dev

## ── WASM ─────────────────────────────────────────────────────────────────

wasm:
	@echo "==> Building WASM (debug)..."
	wasm-pack build --target bundler "$(WASM_CRATE)" --out-dir "$(WASM_OUT)" --dev
	@echo "==> WASM written to $(WASM_OUT)"

wasm-release:
	@echo "==> Building WASM (release)..."
	wasm-pack build --target bundler "$(WASM_CRATE)" --out-dir "$(WASM_OUT)"
	@echo "==> WASM written to $(WASM_OUT)"

## ── Python bindings (for convert.py integration) ─────────────────────────

# Install the Python module into the current virtualenv / system Python.
py-dev:
	@echo "==> Building & installing s57style Python module (maturin develop)..."
	cd "$(PY_CRATE)" && maturin develop --release

# Build a wheel (.whl) that can be installed separately.
py-build:
	@echo "==> Building s57style wheel..."
	cd "$(PY_CRATE)" && maturin build --release

## ── Frontend ─────────────────────────────────────────────────────────────

deps:
	npm install

# Development server — does NOT rebuild WASM automatically; run 'make wasm' first.
dev: deps
	npm run dev

# Production build
build: wasm-release deps
	npm run build

## ── Clean ────────────────────────────────────────────────────────────────

clean:
	rm -rf node_modules dist src/wasm-pkg
