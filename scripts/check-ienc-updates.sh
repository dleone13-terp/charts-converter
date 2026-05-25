#!/usr/bin/env bash
# check-ienc-updates.sh
#
# Checks each country's IENC source for changes since the last run.
# Stable-URL sources (AT, HR, RO, BE) use HTTP Last-Modified comparison.
# Page-scraped sources (DE, RS, BG, PL, CH) fingerprint the sorted list of
# discovered ZIP URLs — if any URL changed (new date embedded in filename),
# the country is queued.
#
# Outputs (written to $GITHUB_OUTPUT):
#   countries_to_run   - JSON array, e.g. ["DE","HR"]
#   has_updates        - "true" or "false"
#
# Environment:
#   FORCE_ALL        - set to "true" to queue every country regardless
#   FORCE_COUNTRIES  - comma-separated list, e.g. "DE,AT"

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STATE_FILE="${REPO_ROOT}/state.json"

FORCE_ALL="${FORCE_ALL:-false}"
FORCE_COUNTRIES="${FORCE_COUNTRIES:-}"

TO_RUN=()
TMP_DIR=$(mktemp -d /tmp/check-ienc-XXXXX)
trap 'rm -rf "$TMP_DIR"' EXIT

# ── Stable-URL countries: HEAD for Last-Modified ──────────────────────
check_last_modified() {
  local code="$1" url="$2"
  local lm stored forced=false
  lm=$(curl -fsI --max-time 20 --retry 3 "$url" \
    | grep -i '^last-modified:' | tr -d '\r' | sed 's/^[Ll]ast-[Mm]odified: //' || true)
  stored=$(python3 -c "
import json
s = json.load(open('${STATE_FILE}'))
print(s['ienc'].get('${code}', {}).get('lastModified') or '')
")
  if [ "$FORCE_ALL" = "true" ] || echo ",$FORCE_COUNTRIES," | grep -q ",${code},"; then forced=true; fi
  if [ "$forced" = "true" ] || [ "$lm" != "$stored" ]; then
    echo "$code" >> "${TMP_DIR}/to_run"
    echo "[check] ${code}: changed (${stored:-never} → ${lm:-unknown})" >&2
  else
    echo "[check] ${code}: up-to-date (${lm})" >&2
  fi
}

# ── Page-scraped countries: fingerprint sorted ZIP URLs ───────────────
check_url_fingerprint() {
  local code="$1" page_url="$2" grep_pattern="$3"
  local fingerprint stored forced=false
  fingerprint=$(curl -fsL --max-time 30 --retry 2 "$page_url" \
    | grep -oE 'href="[^"]*\.zip[^"]*"' \
    | grep -iE "$grep_pattern" \
    | sort | sha256sum | cut -c1-16 || true)
  stored=$(python3 -c "
import json
s = json.load(open('${STATE_FILE}'))
print(s['ienc'].get('${code}', {}).get('urlFingerprint') or '')
")
  if [ "$FORCE_ALL" = "true" ] || echo ",$FORCE_COUNTRIES," | grep -q ",${code},"; then forced=true; fi
  if [ "$forced" = "true" ] || [ -z "$fingerprint" ] || [ "$fingerprint" != "$stored" ]; then
    echo "$code" >> "${TMP_DIR}/to_run"
    echo "[check] ${code}: page changed (fp: ${stored:-never} → ${fingerprint:-unknown})" >&2
  else
    echo "[check] ${code}: up-to-date (fp: ${fingerprint})" >&2
  fi
}

touch "${TMP_DIR}/to_run"

# Stable-URL countries
check_last_modified "AT" "https://www.doris.bmimi.gv.at/fileadmin/content/doris/ECDIS_Download/2W_Edition.zip" &
check_last_modified "HR" "http://www.vodniputovi.hr/enc/dunav/dunav.zip" &

# Page-scraped countries
check_url_fingerprint "RS" "http://www.plovput.rs/electronic-navigational-charts" "IENC.*\.zip" &
check_url_fingerprint "PL" "https://www.szczecin.uzs.gov.pl/?page_id=6092" "IENC.*\.zip" &
check_url_fingerprint "CH" "https://port-of-switzerland.ch/hafenservice/schifffahrtsservice/inland-enc-hochrhein/" "Hochrhein.*\.zip|INLAND.*\.zip" &

# DE: ELWIS requires a browser session — not automatable via curl
# BG: BULRIS has no public direct download links
# BE: Brussels/Wallonia URLs broken or not direct downloads
# RO: ACN download URLs unverified (returned 404)

wait

# Sort and deduplicate (background jobs may race to write)
mapfile -t TO_RUN < <(sort -u "${TMP_DIR}/to_run")

if [ "${#TO_RUN[@]}" -gt 0 ]; then
  TO_RUN_JSON=$(printf '"%s",' "${TO_RUN[@]}" | sed 's/,$//')
  HAS_UPDATES=true
else
  TO_RUN_JSON=""
  HAS_UPDATES=false
fi

OUTPUT_TARGET="${GITHUB_OUTPUT:-/dev/stderr}"
{
  echo "countries_to_run=[${TO_RUN_JSON}]"
  echo "has_updates=${HAS_UPDATES}"
} >> "$OUTPUT_TARGET"

echo "[check] countries to run: [${TO_RUN_JSON}]" >&2
