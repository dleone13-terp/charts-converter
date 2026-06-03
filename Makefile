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
ASSETS_DIR    := $(CURDIR)/assets
VIEWER_DIR    := $(CURDIR)/viewer
SCRIPTS_DIR   := $(CURDIR)/scripts

TILE_PORT     := 8080
VIEWER_PORT   := 5173
SERVER_URL    := http://localhost:$(TILE_PORT)

SIGNALK_CHARTS_DIR ?= $(HOME)/.signalk/charts
SIGNALK_PORT       ?= 3000
SIGNALK_URL        ?= http://localhost:$(SIGNALK_PORT)

.PHONY: all py-module convert generate-styles generate-assets serve serve-stop viewer deploy-signalk download-noaa download-ienc clean-serve

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
	@[ -f "$(OUTPUT_DIR)/$(NAME)-sectors.json" ] && \
	  cp "$(OUTPUT_DIR)/$(NAME)-sectors.json" "$(SERVE_DIR)/data/$(NAME)-sectors.json" || true

## ── Style & sprite generation ─────────────────────────────────────────────────

generate-styles:
	@echo "==> Generating 9 style.jsons and copying sprites/fonts..."
	python3 "$(SCRIPTS_DIR)/generate-styles/generate_styles.py" \
		--server "$(SERVER_URL)" \
		--data-name "$(NAME)" \
		--out-dir "$(SERVE_DIR)"
	@SECTORS="$(SERVE_DIR)/data/$(NAME)-sectors.json"; \
	if [ -f "$$SECTORS" ]; then \
	  echo "==> Merging sector sprites from $$SECTORS..."; \
	  python3 "$(SCRIPTS_DIR)/generate-styles/generate_sector_sprites.py" \
	    "$$SECTORS" "$(SERVE_DIR)/sprites"; \
	fi

# Generate committed release assets: styles (data-name=enc), sprites, fonts.
# Run this whenever the s57style rendering logic changes, then commit assets/.
# Requires: s57style module (make py-module) and the njord sibling repo.
# Sector sprites are chart-specific and added at zip time per chart — NOT here.
generate-assets:
	@echo "==> Generating packaged assets (data-name=enc) → $(ASSETS_DIR)"
	python3 "$(SCRIPTS_DIR)/generate-styles/generate_styles.py" \
		--server "http://localhost:8080" \
		--data-name "enc" \
		--out-dir "$(ASSETS_DIR)"
	@echo "    Done — commit assets/ so CI can include styles/sprites/fonts in release zips."

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

## ── SignalK deployment ───────────────────────────────────────────────────────

# Copy a locally built MBTiles into SignalK's charts dir.
# Usage: make deploy-signalk [NAME=de-encs-styled] [SIGNALK_CHARTS_DIR=~/.signalk/charts]
deploy-signalk:
	@echo "==> Deploying $(NAME).mbtiles → $(SIGNALK_CHARTS_DIR)"
	mkdir -p "$(SIGNALK_CHARTS_DIR)"
	cp "$(OUTPUT_DIR)/$(NAME).mbtiles" "$(SIGNALK_CHARTS_DIR)/$(NAME).mbtiles"
	@echo "    Restart SignalK to pick up the chart."

# Download the latest release zip and extract all charts into SignalK's charts dir.
# Requires: gh CLI authenticated against the repo.
# Usage: make download-noaa DISTRICT=01CGD
DISTRICT ?=
download-noaa:
ifeq ($(DISTRICT),)
	$(error Specify a district: make download-noaa DISTRICT=01CGD)
endif
	@echo "==> Downloading noaa-enc-$(DISTRICT)-styled.zip from noaa-latest..."
	gh release download noaa-latest \
	  --pattern "noaa-enc-$(DISTRICT)-styled.zip" \
	  --output /tmp/noaa-enc-$(DISTRICT)-styled.zip --clobber
	mkdir -p "$(SIGNALK_CHARTS_DIR)"
	unzip -p /tmp/noaa-enc-$(DISTRICT)-styled.zip data/enc.mbtiles \
	  > "$(SIGNALK_CHARTS_DIR)/noaa-enc-$(DISTRICT).mbtiles"
	@echo "    Installed noaa-enc-$(DISTRICT).mbtiles → $(SIGNALK_CHARTS_DIR)"
	@echo "    Restart SignalK to pick up."

# Usage: make download-ienc COUNTRY=de
COUNTRY ?=
download-ienc:
ifeq ($(COUNTRY),)
	$(error Specify a country: make download-ienc COUNTRY=de)
endif
	@echo "==> Downloading eu-ienc-$(COUNTRY)-styled.zip from ienc-latest..."
	gh release download ienc-latest \
	  --pattern "eu-ienc-$(COUNTRY)-styled.zip" \
	  --output /tmp/eu-ienc-$(COUNTRY)-styled.zip --clobber
	mkdir -p "$(SIGNALK_CHARTS_DIR)"
	unzip -p /tmp/eu-ienc-$(COUNTRY)-styled.zip data/enc.mbtiles \
	  > "$(SIGNALK_CHARTS_DIR)/eu-ienc-$(COUNTRY).mbtiles"
	@echo "    Installed eu-ienc-$(COUNTRY).mbtiles → $(SIGNALK_CHARTS_DIR)"
	@echo "    Restart SignalK to pick up."

## ── Viewer ───────────────────────────────────────────────────────────────────

viewer:
	@echo "==> Starting ENC viewer on port $(VIEWER_PORT)..."
	cd "$(VIEWER_DIR)" && npm install && npm run dev

## ── Clean ────────────────────────────────────────────────────────────────────

clean-serve:
	rm -rf "$(SERVE_DIR)"/styles/*.json "$(SERVE_DIR)"/sprites "$(SERVE_DIR)"/fonts
