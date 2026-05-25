#!/usr/bin/env bash
# convert-country-ienc.sh <COUNTRY_CODE>
#
# Downloads all IENC ZIPs for the given country, converts all S-57 .000 cells
# to a single merged MBTiles file using the charts-toolbox container.
#
# All ZIPs for the country are extracted into one flat ENC_DIR. GDAL exports
# each .000 cell into per-layer GeoJSON files (LAYER_CHARTNAME.geojson).
# Tippecanoe merges all GeoJSON files that share a layer name natively via
# multiple -L args — no separate merge step is needed.
#
# Environment:
#   TOOLBOX_IMAGE  (default: ghcr.io/dirkwa/signalk-charts-provider-simple/charts-toolbox:1.0.0)
#
# Output files (written to /tmp/enc-output/):
#   eu-ienc-<COUNTRY>.mbtiles
#   ienc-result-<COUNTRY>.json

set -euo pipefail

COUNTRY="${1:?Usage: convert-country-ienc.sh <COUNTRY_CODE>}"

TOOLBOX="${TOOLBOX_IMAGE:-ghcr.io/dirkwa/signalk-charts-provider-simple/charts-toolbox:1.0.0}"
MIN_ZOOM=9   # IHO Band 7 (River) floor
MAX_ZOOM=18  # IHO Band 9 (River Berth) ceiling

WORK="/tmp/ienc-${COUNTRY}"
OUTPUT="/tmp/enc-output"
ENC_DIR="${WORK}/enc"
GEOJSON_DIR="${WORK}/geojson"
MBTILES_OUT="${OUTPUT}/eu-ienc-${COUNTRY}.mbtiles"
RESULT_JSON="${OUTPUT}/ienc-result-${COUNTRY}.json"
URLS_FILE="${WORK}/source-urls.txt"

mkdir -p "$ENC_DIR" "$GEOJSON_DIR" "$OUTPUT" "${WORK}/zips"

echo "=== Converting ${COUNTRY} IENC ==="
echo "    Image: ${TOOLBOX}"
docker pull "$TOOLBOX"

# ── Country-specific download logic ──────────────────────────────────
download_zips() {
  case "$COUNTRY" in

    DE)
      # ELWIS requires a JavaScript browser session — curl downloads return HTML,
      # not ZIP files. DE is not automatable via simple HTTP. Skip gracefully.
      echo "Warning: DE (ELWIS) requires a browser session and cannot be downloaded automatically."
      ;;


    AT)
      URL="https://www.doris.bmimi.gv.at/fileadmin/content/doris/ECDIS_Download/2W_Edition.zip"
      echo "$URL" > "$URLS_FILE"
      curl -fL --retry 3 --progress-bar -o "${WORK}/zips/AT_Danube.zip" "$URL"
      ;;

    HR)
      printf '%s\n' \
        "http://www.vodniputovi.hr/enc/dunav/dunav.zip" \
        "http://www.vodniputovi.hr/enc/sava/sava.zip" \
        "http://www.vodniputovi.hr/enc/drava/drava.zip" | tee "$URLS_FILE" \
        | while read -r url; do
            curl -fL --retry 3 --progress-bar -o "${WORK}/zips/$(basename "$url")" "$url"
          done
      ;;

    RS)
      BASE="http://www.plovput.rs"
      # plovput.rs can be unreachable from GH Actions runners; || true prevents
      # a DNS/connection failure from crashing the whole script — ZIP_COUNT check
      # below will then fail the job gracefully so it's retried next run.
      (curl -fsL --max-time 30 "http://www.plovput.rs/electronic-navigational-charts" \
        | grep -oE 'href="[^"]*IENC[^"]*\.zip"' \
        | grep -oE '"[^"]*"' | tr -d '"' | sort -u \
        | tee "$URLS_FILE" \
        | while read -r path; do
            url="${BASE}${path}"
            curl -fL --retry 3 --progress-bar -o "${WORK}/zips/$(basename "$path")" "$url"
          done) || echo "Warning: RS (plovput.rs) download failed — site may be unreachable"
      ;;

    BG)
      # BULRIS page has no parseable direct ZIP links (JS-rendered). No working
      # public download URL found. BG is not automatable via simple scraping.
      echo "Warning: BG (BULRIS) has no direct download links on its public page."
      ;;

    PL)
      BASE="https://www.szczecin.uzs.gov.pl"
      curl -fsL --max-time 30 "https://www.szczecin.uzs.gov.pl/?page_id=6092" \
        | grep -oE 'href="[^"]*IENC[^"]*\.zip"' \
        | grep -oE '"[^"]*"' | tr -d '"' | sort -u \
        | tee "$URLS_FILE" \
        | while read -r path; do
            [[ "$path" =~ ^https?:// ]] && url="$path" || url="${BASE}${path}"
            curl -fL --retry 3 --progress-bar -o "${WORK}/zips/$(basename "$path")" "$url"
          done
      ;;

    CH)
      curl -fsL --max-time 30 \
        "https://port-of-switzerland.ch/hafenservice/schifffahrtsservice/inland-enc-hochrhein/" \
        | grep -oE 'href="[^"]*(Hochrhein|INLAND)[^"]*\.zip"' \
        | grep -oE '"[^"]*"' | tr -d '"' | head -1 \
        | tee "$URLS_FILE" \
        | while read -r url; do
            [[ "$url" =~ ^https?:// ]] || url="https://port-of-switzerland.ch${url}"
            curl -fL --retry 3 --progress-bar -o "${WORK}/zips/$(basename "$url")" "$url"
          done
      ;;

    RO)
      # ACN (acn.ro) URLs could not be confirmed — guessed paths returned 404.
      # RO requires manual URL verification before it can be automated.
      echo "Warning: RO (ACN) download URLs are unverified and returned 404."
      ;;

    BE)
      # Brussels ENC_ROOT.zip URL returned 404; Wallonia URL is a catalogue page,
      # not a direct download. BE requires URL verification before automation.
      echo "Warning: BE download URLs are broken (404) and require manual verification."
      ;;

    *)
      echo "ERROR: Unknown country code: ${COUNTRY}"
      exit 1
      ;;
  esac
}

# ── Step 1: Download & extract all ZIPs → ENC_DIR ────────────────────
echo "[1/3] Downloading ${COUNTRY} IENC ZIPs..."
download_zips

ZIP_COUNT=$(find "${WORK}/zips" -name "*.zip" -type f | wc -l)
echo "  Downloaded ${ZIP_COUNT} ZIP(s)"
if [ "$ZIP_COUNT" -eq 0 ]; then
  echo "ERROR: no ZIPs downloaded for ${COUNTRY}"
  exit 1
fi

for zip in "${WORK}/zips"/*.zip; do
  [ -f "$zip" ] || continue
  unzip -qo "$zip" -d "$ENC_DIR" || echo "Warning: failed to extract $zip"
  rm -f "$zip"
done
rmdir "${WORK}/zips" 2>/dev/null || true

ENC_COUNT=$(find "$ENC_DIR" -name "*.000" ! -name "._*" -type f | wc -l)
echo "  Found ${ENC_COUNT} ENC cells (.000 files)"
if [ "$ENC_COUNT" -eq 0 ]; then
  echo "ERROR: no .000 files found after extraction"
  exit 1
fi

# ── Step 2: GDAL export → GeoJSON (inside container) ─────────────────
# Parallel branch from signalk-charts-provider-simple buildExportScript()
# with multiFile=true and parallelism=$(nproc): per-layer ogr2ogr calls within
# each .000 cell are fanned out via xargs -P so all CPUs are used.
echo "[2/3] GDAL export: ${ENC_COUNT} cells → FlatGeobuf..."

GDAL_SCRIPT=$(mktemp /tmp/gdal-XXXXX.sh)
cat > "$GDAL_SCRIPT" <<'GDAL'
#!/bin/bash
set -e
: > /output/.export-errors.log
count=$(find /input -name '*.000' ! -name '._*' -type f | wc -l)
i=0
find /input -name '*.000' ! -name '._*' -type f -print0 | while IFS= read -r -d '' enc; do
  i=$((i + 1))
  name=$(basename "$enc" .000)
  echo "PROGRESS: Processing $name ($i/$count)"
  layers=$(ogrinfo -so "$enc" 2>>/output/.export-errors.log | grep -E '^[0-9]+:' | awk -F': ' '{print $2}' | awk '{print $1}')
  printf '%s\n' $layers | xargs -P "$(nproc)" -I '{}' sh -c '
    layer="$1" enc="$2" name="$3"
    case "$layer" in DSID|C_AGGR|C_ASSO|Generic) exit 0 ;; esac
    outname="${layer}_${name}"
    if [ "$layer" = "SOUNDG" ]; then
      ogr2ogr -f FlatGeobuf -oo SPLIT_MULTIPOINT=YES -oo ADD_SOUNDG_DEPTH=YES \
        "/output/${outname}.fgb" "$enc" "$layer" 2>>/output/.export-errors.log || true
    else
      ogr2ogr -f FlatGeobuf "/output/${outname}.fgb" "$enc" "$layer" \
        2>>/output/.export-errors.log || true
    fi
  ' _ '{}' "$enc" "$name"
done
echo "PROGRESS: Export complete"
GDAL
chmod +x "$GDAL_SCRIPT"

docker run --rm \
  --user "$(id -u):$(id -g)" \
  -v "${ENC_DIR}:/input:ro" \
  -v "${GEOJSON_DIR}:/output:rw" \
  -v "${GDAL_SCRIPT}:/run-gdal-export.sh:ro" \
  "$TOOLBOX" \
  bash /run-gdal-export.sh

rm -f "$GDAL_SCRIPT"

rm -rf "$ENC_DIR"

FGB_COUNT=$(find "$GEOJSON_DIR" -name "*.fgb" ! -name ".*" -type f | wc -l)
echo "  Produced ${FGB_COUNT} FlatGeobuf files"
if [ "$FGB_COUNT" -eq 0 ]; then
  echo "ERROR: GDAL export produced no FlatGeobuf output"
  if [ -f "${GEOJSON_DIR}/.export-errors.log" ]; then
    echo "--- export errors ---"
    tail -20 "${GEOJSON_DIR}/.export-errors.log"
  fi
  exit 1
fi

# ── Step 3: Tippecanoe → country MBTiles (inside container) ───────────
# Build -L LAYER:/input/FILE.fgb args for every FlatGeobuf file.
# Layer name = filename with trailing _CHARTID stripped when the suffix
# contains a digit (mirrors the plugin's tailHasDigit logic).
# Tippecanoe merges features from multiple -L args that share a layer name,
# so all cells from all ZIPs are combined into one vector layer per type.
echo "[3/3] Running tippecanoe (z${MIN_ZOOM}–z${MAX_ZOOM})..."

LAYER_ARGS=""
while IFS= read -r f; do
  filename=$(basename "$f" .fgb)
  suffix="${filename##*_}"
  if echo "$suffix" | grep -qE '[0-9]'; then
    layer="${filename%_*}"
  else
    layer="$filename"
  fi
  LAYER_ARGS="${LAYER_ARGS} -L ${layer}:/input/$(basename "$f")"
done < <(find "$GEOJSON_DIR" -name "*.fgb" ! -name ".*" -type f | sort)

TIPP_SCRIPT=$(mktemp /tmp/tipp-XXXXX.sh)
cat > "$TIPP_SCRIPT" <<TIPP
#!/bin/bash
tippecanoe \\
  -o /output/country.mbtiles \\
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

mv "${OUTPUT}/country.mbtiles" "$MBTILES_OUT"

# ── Patch MBTiles metadata ────────────────────────────────────────────
# sqlite3 CLI is not in the toolbox image; use the host runner's python3
# (the .mbtiles file lives on the host filesystem at $MBTILES_OUT).
python3 - <<PY
import sqlite3
conn = sqlite3.connect("${MBTILES_OUT}")
conn.executemany("INSERT OR REPLACE INTO metadata VALUES(?,?)", [
    ("name", "IENC ${COUNTRY}"),
    ("description", "European Inland ENC - ${COUNTRY}"),
    ("type", "S-57"),
])
conn.commit()
conn.close()
PY

rm -rf "$WORK"

SIZE=$(stat -c%s "$MBTILES_OUT")
echo "=== Done: $(du -sh "$MBTILES_OUT" | cut -f1) ==="

# ── Write result JSON for the publish job ─────────────────────────────
URL_FP=""
if [ -f "$URLS_FILE" ]; then
  URL_FP=$(sort "$URLS_FILE" | sha256sum | cut -c1-16)
fi

LAST_MODIFIED=$(curl -fsI --max-time 10 "$(head -1 "$URLS_FILE" 2>/dev/null || echo '')" \
  2>/dev/null | grep -i '^last-modified:' | tr -d '\r' | sed 's/^[Ll]ast-[Mm]odified: //' || true)

SAFE_URL_FP=$(printf '%s' "$URL_FP" | sed 's/"/\\"/g')
SAFE_LM=$(printf '%s' "$LAST_MODIFIED" | sed 's/"/\\"/g')

cat > "$RESULT_JSON" <<JSON
{
  "country": "${COUNTRY}",
  "urlFingerprint": "${SAFE_URL_FP}",
  "lastModified": "${SAFE_LM}",
  "lastConverted": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "success": true,
  "sizeBytes": ${SIZE}
}
JSON
