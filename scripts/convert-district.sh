#!/usr/bin/env bash
# convert-district.sh <DISTRICT_ID>
#
# Downloads the NOAA ENC ZIP for one Coast Guard District, converts all
# S-57 .000 cells to a single merged MBTiles file, and writes a result
# JSON so the workflow's publish job can update state.json.
#
# The toolbox container (GDAL + tippecanoe + tile-join) does all the
# heavy lifting.  The host-side script only:
#   1. Fetches the current Last-Modified header (for state tracking)
#   2. Downloads and unzips the NOAA bundle
#   3. Runs the GDAL export inside the container (same bash script the
#      signalk-charts-provider-simple plugin uses, verbatim)
#   4. Builds tippecanoe -L LAYER:FILE args by stripping chart-ID suffixes
#      from filenames (tippecanoe merges same-named layers natively)
#   5. Runs tippecanoe inside the container
#   6. Patches MBTiles metadata via sqlite3 inside the container
#
# Environment:
#   TOOLBOX_IMAGE  (default: ghcr.io/dirkwa/signalk-charts-provider-simple/charts-toolbox:1.0.0)
#   MIN_ZOOM       (default: 4)
#   MAX_ZOOM       (default: 16)
#
# Output files (written to /tmp/enc-output/):
#   noaa-enc-<DISTRICT>.mbtiles
#   district-result-<DISTRICT>.json

set -euo pipefail

DISTRICT="${1:?Usage: convert-district.sh <DISTRICT_ID>}"

TOOLBOX="${TOOLBOX_IMAGE:-ghcr.io/dirkwa/signalk-charts-provider-simple/charts-toolbox:1.0.0}"
MIN_ZOOM="${MIN_ZOOM:-4}"
MAX_ZOOM="${MAX_ZOOM:-16}"

ZIP_URL="https://charts.noaa.gov/ENCs/${DISTRICT}_ENCs.zip"
WORK="/tmp/enc-${DISTRICT}"
OUTPUT="/tmp/enc-output"
ENC_DIR="${WORK}/enc"
GEOJSON_DIR="${WORK}/geojson"
MBTILES_OUT="${OUTPUT}/noaa-enc-${DISTRICT}.mbtiles"
RESULT_JSON="${OUTPUT}/district-result-${DISTRICT}.json"

mkdir -p "$ENC_DIR" "$GEOJSON_DIR" "$OUTPUT"

# ── Step 0: Pull image + fetch current Last-Modified ─────────────────
echo "=== Converting ${DISTRICT} ==="
echo "    Image: ${TOOLBOX}"
docker pull "$TOOLBOX"

LAST_MODIFIED=$(curl -fsI --max-time 20 --retry 3 "$ZIP_URL" \
  | grep -i '^last-modified:' | tr -d '\r' | sed 's/^[Ll]ast-[Mm]odified: //' || true)
echo "    Last-Modified: ${LAST_MODIFIED:-unknown}"

# ── Step 1: Download & extract district ZIP ───────────────────────────
echo "[1/3] Downloading ${DISTRICT} ZIP..."
curl -fL --retry 3 --progress-bar \
  "$ZIP_URL" \
  -o "${WORK}/district.zip"
echo "  Downloaded: $(du -sh "${WORK}/district.zip" | cut -f1)"

unzip -q "${WORK}/district.zip" -d "$ENC_DIR"
rm -f "${WORK}/district.zip"

ENC_COUNT=$(find "$ENC_DIR" -name "*.000" ! -name "._*" -type f | wc -l)
echo "  Found ${ENC_COUNT} ENC cells (.000 files)"
if [ "$ENC_COUNT" -eq 0 ]; then
  echo "ERROR: no .000 files found after extraction"
  exit 1
fi

# ── Step 2: GDAL export → GeoJSON (inside container) ─────────────────
# Verbatim from signalk-charts-provider-simple buildExportScript()
# with multiFile=true (outputs LAYER_CHARTNAME.geojson) and parallelism=1.
# Skip layers: DSID, C_AGGR, C_ASSO, Generic (same as the plugin).
echo "[2/3] GDAL export: ${ENC_COUNT} cells → GeoJSON..."

docker run --rm \
  --user "$(id -u):$(id -g)" \
  -v "${ENC_DIR}:/input:ro" \
  -v "${GEOJSON_DIR}:/output:rw" \
  "$TOOLBOX" \
  bash -c '
set -e
: > /output/.export-errors.log
count=$(find /input -name '"'"'*.000'"'"' ! -name '"'"'._*'"'"' -type f | wc -l)
i=0
find /input -name '"'"'*.000'"'"' ! -name '"'"'._*'"'"' -type f -print0 | while IFS= read -r -d '"'"''"'"' enc; do
  i=$((i + 1))
  name=$(basename "$enc" .000)
  echo "PROGRESS: Processing $name ($i/$count)"
  layers=$(ogrinfo -so "$enc" 2>>/output/.export-errors.log | grep -E '"'"'^[0-9]+:'"'"' | awk -F'"'"': '"'"' '"'"'{print $2}'"'"' | awk '"'"'{print $1}'"'"')
  for layer in $layers; do
    case "$layer" in DSID|C_AGGR|C_ASSO|Generic) continue ;; esac
    outname="${layer}_${name}"
    if [ "$layer" = "SOUNDG" ]; then
      ogr2ogr -f GeoJSON -oo SPLIT_MULTIPOINT=YES -oo ADD_SOUNDG_DEPTH=YES \
        "/output/$outname.geojson" "$enc" "$layer" 2>>/output/.export-errors.log || true
    else
      ogr2ogr -f GeoJSON "/output/$outname.geojson" "$enc" "$layer" \
        2>>/output/.export-errors.log || true
    fi
  done
done
echo "PROGRESS: Export complete"
'

rm -rf "$ENC_DIR"

GEOJSON_COUNT=$(find "$GEOJSON_DIR" -name "*.geojson" ! -name ".*" -type f | wc -l)
echo "  Produced ${GEOJSON_COUNT} GeoJSON files"
if [ "$GEOJSON_COUNT" -eq 0 ]; then
  echo "ERROR: GDAL export produced no GeoJSON output"
  if [ -f "${GEOJSON_DIR}/.export-errors.log" ]; then
    echo "--- export errors ---"
    tail -20 "${GEOJSON_DIR}/.export-errors.log"
  fi
  exit 1
fi

# ── Step 3: Tippecanoe → district MBTiles (inside container) ──────────
# Build -L LAYER:/input/FILE.geojson args for every GeoJSON file.
# Layer name = filename with trailing _CHARTID stripped when the suffix
# contains a digit (mirrors the plugin's tailHasDigit logic).
# Tippecanoe merges features from multiple -L args that share a layer name,
# so no pre-merge step is needed.
echo "[3/3] Running tippecanoe (z${MIN_ZOOM}–z${MAX_ZOOM})..."

LAYER_ARGS=""
while IFS= read -r f; do
  filename=$(basename "$f" .geojson)
  suffix="${filename##*_}"
  if echo "$suffix" | grep -qE '[0-9]'; then
    layer="${filename%_*}"
  else
    layer="$filename"
  fi
  LAYER_ARGS="${LAYER_ARGS} -L ${layer}:/input/$(basename "$f")"
done < <(find "$GEOJSON_DIR" -name "*.geojson" ! -name ".*" -type f | sort)

# Write the tippecanoe invocation to a temp script so shell quoting in the
# layer args survives the docker run boundary without escaping issues.
TIPP_SCRIPT=$(mktemp /tmp/tipp-XXXXX.sh)
cat > "$TIPP_SCRIPT" <<TIPP
#!/bin/bash
tippecanoe \\
  -o /output/district.mbtiles \\
  -Z ${MIN_ZOOM} -z ${MAX_ZOOM} \\
  --no-tile-size-limit --no-feature-limit \\
  --detect-shared-borders --no-simplification \\
  --no-tiny-polygon-reduction --buffer=80 --force \\
  ${LAYER_ARGS}
TIPP
chmod +x "$TIPP_SCRIPT"

docker run --rm \
  --user "$(id -u):$(id -g)" \
  -v "${GEOJSON_DIR}:/input:ro" \
  -v "${OUTPUT}:/output:rw" \
  -v "${TIPP_SCRIPT}:/run-tippecanoe.sh:ro" \
  -e "TIPPECANOE_MAX_THREADS=$(nproc)" \
  "$TOOLBOX" \
  bash /run-tippecanoe.sh

rm -f "$TIPP_SCRIPT"

mv "${OUTPUT}/district.mbtiles" "$MBTILES_OUT"

# ── Patch MBTiles metadata ────────────────────────────────────────────
docker run --rm \
  --user "$(id -u):$(id -g)" \
  -v "${OUTPUT}:/output:rw" \
  "$TOOLBOX" \
  bash -c "
sqlite3 /output/noaa-enc-${DISTRICT}.mbtiles \"
  INSERT OR REPLACE INTO metadata VALUES('name','NOAA ENC ${DISTRICT}');
  INSERT OR REPLACE INTO metadata VALUES('description','NOAA Electronic Navigational Charts - ${DISTRICT}');
  INSERT OR REPLACE INTO metadata VALUES('type','S-57');
\"
"

rm -rf "$WORK"

SIZE=$(stat -c%s "$MBTILES_OUT")
echo "=== Done: $(du -sh "$MBTILES_OUT" | cut -f1) ==="

# ── Write result JSON for the publish job ─────────────────────────────
# Last-Modified value is passed in from check-updates; single-quote any
# double quotes so the JSON stays valid.
SAFE_LM=$(printf '%s' "$LAST_MODIFIED" | sed 's/"/\\"/g')
cat > "$RESULT_JSON" <<JSON
{
  "district": "${DISTRICT}",
  "lastModified": "${SAFE_LM}",
  "lastConverted": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "success": true,
  "sizeBytes": ${SIZE}
}
JSON
