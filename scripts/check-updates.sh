#!/usr/bin/env bash
# check-updates.sh
#
# HEAD-requests each NOAA district ZIP and compares the Last-Modified header
# against the value stored in state.json.  Districts whose header changed (or
# was never recorded) are written to GITHUB_OUTPUT as a JSON matrix so the
# workflow's convert job knows what to run.
#
# Outputs (written to $GITHUB_OUTPUT):
#   districts_to_run   - JSON array, e.g. ["01CGD","17CGD"]
#   has_updates        - "true" or "false"
#
# Environment:
#   FORCE_ALL          - set to "true" to queue every district regardless
#   FORCE_DISTRICTS    - comma-separated list of districts to force, e.g. "01CGD,09CGD"

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STATE_FILE="${REPO_ROOT}/state.json"

FORCE_ALL="${FORCE_ALL:-false}"
FORCE_DISTRICTS="${FORCE_DISTRICTS:-}"

declare -A URLS=(
  [01CGD]="https://charts.noaa.gov/ENCs/01CGD_ENCs.zip"
  [05CGD]="https://charts.noaa.gov/ENCs/05CGD_ENCs.zip"
  [07CGD]="https://charts.noaa.gov/ENCs/07CGD_ENCs.zip"
  [08CGD]="https://charts.noaa.gov/ENCs/08CGD_ENCs.zip"
  [09CGD]="https://charts.noaa.gov/ENCs/09CGD_ENCs.zip"
  [11CGD]="https://charts.noaa.gov/ENCs/11CGD_ENCs.zip"
  [13CGD]="https://charts.noaa.gov/ENCs/13CGD_ENCs.zip"
  [14CGD]="https://charts.noaa.gov/ENCs/14CGD_ENCs.zip"
  [17CGD]="https://charts.noaa.gov/ENCs/17CGD_ENCs.zip"
)

# Fan out HEAD requests in parallel so all 9 finish in ~one round-trip time.
declare -A PIDS
declare -A TIMESTAMPS
TMP_DIR=$(mktemp -d /tmp/check-updates-XXXXX)
trap 'rm -rf "$TMP_DIR"' EXIT

for district in "${!URLS[@]}"; do
  (
    curl -fsI --max-time 20 --retry 3 "${URLS[$district]}" \
      | grep -i '^last-modified:' \
      | tr -d '\r' \
      | sed 's/^[Ll]ast-[Mm]odified: //' \
      || true
  ) > "${TMP_DIR}/${district}.lm" &
  PIDS[$district]=$!
done

for district in "${!PIDS[@]}"; do
  wait "${PIDS[$district]}" || true
  TIMESTAMPS[$district]=$(cat "${TMP_DIR}/${district}.lm" 2>/dev/null || true)
done

# Compare each timestamp against state.json and build the list to run.
TO_RUN=()
for district in $(echo "${!URLS[@]}" | tr ' ' '\n' | sort); do
  lm="${TIMESTAMPS[$district]:-}"

  stored=$(python3 - <<PY
import json
state = json.load(open('${STATE_FILE}'))
print(state['districts'].get('${district}', {}).get('lastModified') or '')
PY
)

  forced=false
  if [ "$FORCE_ALL" = "true" ]; then forced=true; fi
  if echo ",$FORCE_DISTRICTS," | grep -q ",${district},"; then forced=true; fi

  if [ "$forced" = "true" ] || [ "$lm" != "$stored" ]; then
    TO_RUN+=("$district")
    echo "[check] ${district}: changed (${stored:-never} → ${lm:-unknown})" >&2
  else
    echo "[check] ${district}: up-to-date (${lm})" >&2
  fi
done

if [ "${#TO_RUN[@]}" -gt 0 ]; then
  TO_RUN_JSON=$(printf '"%s",' "${TO_RUN[@]}" | sed 's/,$//')
  HAS_UPDATES=true
else
  TO_RUN_JSON=""
  HAS_UPDATES=false
fi

OUTPUT_TARGET="${GITHUB_OUTPUT:-/dev/stderr}"
{
  echo "districts_to_run=[${TO_RUN_JSON}]"
  echo "has_updates=${HAS_UPDATES}"
} >> "$OUTPUT_TARGET"

echo "[check] districts to run: [${TO_RUN_JSON}]" >&2
