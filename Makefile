# ENC Charts — root Makefile
#
# Prerequisites:
#   docker             (for tileserver-gl)
#   python3 + pip      (for convert.py and generate_styles.py)
#   maturin            (pip install maturin)
#   tippecanoe, ogr2ogr (for conversion)
#   node >= 18         (for viewer)
#
# Quick start:
#   make py-module          # build & install s57style Python wheel
#   make convert INPUT=encs/DE_ENCs\(1\).zip
#   make generate-styles    # create 9 style.jsons + copy sprites/fonts
#   make serve              # start tileserver-gl (Docker)
#   make viewer             # start Vite dev server

BOAT_DIR      := $(realpath $(CURDIR)/..)
STYLE_RUST    := $(BOAT_DIR)/openenc-styling-rust
PY_CRATE      := $(STYLE_RUST)/crates/s57style-python
WHEEL_DIR     := $(STYLE_RUST)/target/wheels

SERVE_DIR     := $(CURDIR)/serve
VIEWER_DIR    := $(CURDIR)/viewer
SCRIPTS_DIR   := $(CURDIR)/scripts

TILE_PORT     := 8080
VIEWER_PORT   := 5173
SERVER_URL    := http://localhost:$(TILE_PORT)

.PHONY: all py-module convert generate-styles serve serve-stop viewer clean-serve

all: generate-styles

## ── Python s57style module ────────────────────────────────────────────────────

py-module:
	@echo "==> Building s57style Python wheel..."
	cd "$(PY_CRATE)" && maturin build --release
	@wheel=$$(ls "$(WHEEL_DIR)"/s57style-*.whl | tail -1); \
	echo "==> Installing $$wheel"; \
	pip install "$$wheel" --break-system-packages --force-reinstall

## ── Conversion ───────────────────────────────────────────────────────────────

# Usage: make convert INPUT=encs/DE_ENCs\(1\).zip [OUTPUT_DIR=output] [NAME=de-encs-styled]
INPUT      ?= encs/DE_ENCs\(1\).zip
OUTPUT_DIR ?= output
NAME       ?= de-encs-styled

convert:
	@echo "==> Converting $(INPUT) → $(OUTPUT_DIR)/$(NAME).mbtiles (with styling)..."
	python3 scripts/convert-enc/convert.py "$(INPUT)" "$(OUTPUT_DIR)" "$(NAME)" --style
	@echo "==> Copying to serve/data/..."
	cp "$(OUTPUT_DIR)/$(NAME).mbtiles" "$(SERVE_DIR)/data/$(NAME).mbtiles"

## ── Style & sprite generation ─────────────────────────────────────────────────

generate-styles:
	@echo "==> Generating 9 style.jsons and copying sprites/fonts..."
	python3 "$(SCRIPTS_DIR)/generate-styles/generate_styles.py" \
		--server "$(SERVER_URL)" \
		--data-name "$(NAME)" \
		--out-dir "$(SERVE_DIR)"

## ── Tileserver-gl (Docker) ───────────────────────────────────────────────────

serve:
	@echo "==> Starting tileserver-gl on port $(TILE_PORT)..."
	docker run --rm -d \
		--name enc-tileserver \
		-v "$(SERVE_DIR)":/data \
		-p "$(TILE_PORT)":8080 \
		maptiler/tileserver-gl --config /data/config.json
	@echo "    → http://localhost:$(TILE_PORT)"

serve-stop:
	docker stop enc-tileserver 2>/dev/null || true

## ── Viewer ───────────────────────────────────────────────────────────────────

viewer:
	@echo "==> Starting ENC viewer on port $(VIEWER_PORT)..."
	cd "$(VIEWER_DIR)" && npm install && npm run dev

## ── Clean ────────────────────────────────────────────────────────────────────

clean-serve:
	rm -rf "$(SERVE_DIR)"/styles/*.json "$(SERVE_DIR)"/sprites "$(SERVE_DIR)"/fonts
