#!/usr/bin/env python3
"""
S-57 ENC → MBTiles converter.

Designed to run inside the charts-toolbox container (ogr2ogr + tippecanoe
are called as local binaries). See Dockerfile for the container definition.

Pipeline:
  unzip → ogr2ogr (S-57 → GeoJSONSeq, parallel per layer) →
  stream-consolidate per layer → tippecanoe (parallel per band) →
  tile-join → MBTiles

Usage (inside container):
    python3 convert.py <zip-or-dir> [output-dir] [output-name]

    The first argument may be a ZIP archive or a directory of already-extracted
    ENC files.  The directory form is used for IENC sources where multiple ZIPs
    are downloaded and extracted into one folder before the container runs.

Usage (via Docker — single ZIP):
    docker run --rm --user "$(id -u):$(id -g)" \\
      -v /path/to/input.zip:/data/input.zip:ro \\
      -v /path/to/output:/data/output \\
      enc-converter /data/input.zip /data/output [name]

Usage (via Docker — pre-extracted directory):
    docker run --rm --user "$(id -u):$(id -g)" \\
      -v /path/to/enc-dir:/data/enc:ro \\
      -v /path/to/output:/data/output \\
      enc-converter /data/enc /data/output [name]

    --user is required so the container writes output files as the host user.
"""
from __future__ import annotations

import json
import math
import os
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time
import zipfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Optional

EXPORT_ERROR_FILE_BASENAME = '.export-errors.log'

# RFC 8142 record separator — triggers tippecanoe parallel reads automatically
_RS = '\x1e'

# ── IHO band tables — verbatim from s57-band.ts ──────────────────────────────

BAND_MAX_ZOOM: dict[int, int] = {
    1: 8,   # Overview      ~1:3,500,000
    2: 10,  # General       ~1:700,000
    3: 12,  # Coastal       ~1:90,000
    4: 14,  # Approach      ~1:22,000
    5: 16,  # Harbour       ~1:8,000
    6: 18,  # Berthing      ~1:3,000
    7: 14,  # River         ~1:10,000
    8: 16,  # River Harbour
    9: 18,  # River Berth
}
BAND_MIN_ZOOM: dict[int, int] = {
    1: 4, 2: 6, 3: 8, 4: 10, 5: 12, 6: 14, 7: 9, 8: 13, 9: 15,
}

# S-57 properties dropped before tiling — they add tile weight with no rendering value.
_SKIP_PROPERTIES: frozenset[str] = frozenset({
    # Internal S-57 / GDAL record-keeping fields
    'RCID', 'RVER', 'PRIM', 'GRUP', 'OBJL',  # object record metadata
    'AGEN',                                    # producing agency (numeric code)
    'FIDN', 'FIDS',                            # feature identifier number/subdivision
    'LNAM',                                    # long-name hex identifier
    'LNAM_REFS', 'FFPT_RIND',                 # cross-object relational links
    # Data provenance / source quality (who surveyed it, when)
    'SORIND', 'SORDAT',                        # source indication and date
    'SURSTA', 'SUREND',                        # survey start/end dates
    # Verbose free text (not rendered by a tile client)
    'INFORM', 'NINFOM',                        # English / national-language info blobs
    'TXTDSC', 'NTXTDS',                        # text description (English / national)
    # Display scale — handled by the band pipeline and SCAMIN logic above
    'SCAMAX',
    # Magnetic variation fields (not for nautical chart display)
    'RYRMGV', 'VALACM',                        # year of MagVar / annual change
})

# ── Band detection — verbatim from s57-band.ts ────────────────────────────────

def detect_enc_band(filename: str) -> Optional[int]:
    base = re.sub(r'\.[^.]+$', '', filename)
    m = re.match(r'^(?:[A-Z][A-Z0-9]|[A-Z0-9][A-Z])(\d)', base)
    if not m:
        return None
    band = int(m.group(1))
    return band if 1 <= band <= 9 else None

def _extract_chart_id(filename: str) -> str:
    base = re.sub(r'\.[^.]+$', '', os.path.basename(filename))
    underscore = base.rfind('_')
    if underscore == -1:
        return base
    tail = base[underscore + 1:]
    return tail if re.search(r'\d', tail) else base

def highest_band_for_files(filenames: list[str]) -> Optional[int]:
    highest: Optional[int] = None
    for f in filenames:
        b = detect_enc_band(_extract_chart_id(f))
        if b is not None and (highest is None or b > highest):
            highest = b
    return highest

def band_clamped_maxzoom(enc_files: list[str], user_maxzoom: int) -> dict:
    bands = sorted({
        b for f in enc_files
        if (b := detect_enc_band(os.path.basename(f))) is not None
    })
    if not bands:
        return {'effective': user_maxzoom, 'highest_band': None, 'bands': []}
    highest_band = bands[-1]
    band_ceiling = BAND_MAX_ZOOM.get(highest_band)
    effective = min(user_maxzoom, band_ceiling) if band_ceiling is not None else user_maxzoom
    return {'effective': effective, 'highest_band': highest_band, 'bands': bands}

# ── SCAMIN → zoom ─────────────────────────────────────────────────────────────
# Web Mercator scale at zoom 0 (equator, 256 px tiles, 96 dpi).
_SCAMIN_SCALE_Z0 = 591_657_527.59

def scamin_to_minzoom(scamin) -> Optional[int]:
    """Convert an S-57 SCAMIN value to a tippecanoe minzoom.

    SCAMIN is the minimum scale denominator at which the feature is visible
    (encoded as denominator − 1, so 179999 → 1:180 000).  Returns None when
    SCAMIN is absent, zero, or non-numeric (meaning "always visible").
    """
    try:
        v = int(scamin)
    except (TypeError, ValueError):
        return None
    if v <= 0:
        return None
    return math.floor(math.log2(_SCAMIN_SCALE_Z0 / v))

def group_cells_by_band(enc_files: list[str]) -> dict:
    by_band: dict[int, list[str]] = {}
    unbanded: list[str] = []
    for f in enc_files:
        band = detect_enc_band(os.path.basename(f))
        if band is None:
            unbanded.append(f)
        else:
            by_band.setdefault(band, []).append(f)
    return {'by_band': by_band, 'unbanded': unbanded, 'bands': sorted(by_band)}

# ── Bash export script ────────────────────────────────────────────────────────
# GeoJSONSeq with COORDINATE_PRECISION=6 (~11 cm), -makevalid for topology
# repair, LIST_AS_STRING so OGR list attributes arrive as comma strings.
# .format() with named kwargs; {{ }} produces literal { } in bash output.

_SKIP_LAYERS = 'DSID|C_AGGR|C_ASSO|Generic'

_SEQ_TMPL = """\
set -e
: > {err_log}
count=$(find {in_dir} -name '*.000' ! -name '._*' -type f | wc -l)
i=0
find {in_dir} -name '*.000' ! -name '._*' -type f -print0 | while IFS= read -r -d '' enc; do
  i=$((i + 1))
  name=$(basename "$enc" .000)
  echo "PROGRESS: Processing $name ($i/$count)"
  layers=$(ogrinfo -so "$enc" 2>>{err_log} | grep -E '^[0-9]+:' | awk -F': ' '{{print $2}}' | awk '{{print $1}}')
  for layer in $layers; do
    case "$layer" in {skip}) continue ;; esac
    outname={outname}
    if [ "$layer" = "SOUNDG" ]; then
      ogr2ogr -f GeoJSONSeq -makevalid -lco COORDINATE_PRECISION=6 \\
        -oo SPLIT_MULTIPOINT=YES -oo ADD_SOUNDG_DEPTH=YES -oo LIST_AS_STRING=YES \\
        "{out_dir}/$outname.geojsons" "$enc" "$layer" 2>>{err_log} || true
    else
      ogr2ogr -f GeoJSONSeq -makevalid -lco COORDINATE_PRECISION=6 -oo LIST_AS_STRING=YES \\
        "{out_dir}/$outname.geojsons" "$enc" "$layer" 2>>{err_log} || true
    fi
  done
done
echo "PROGRESS: Export complete"
"""

# Inside sh -c '...', single quotes mean no bash line-continuation.
# Python's \ + newline line-continuation makes these into single long lines,
# which is valid bash. Keep ogr2ogr invocations on one line here.
_PAR_TMPL = """\
set -e
: > {err_log}
count=$(find {in_dir} -name '*.000' ! -name '._*' -type f | wc -l)
i=0
find {in_dir} -name '*.000' ! -name '._*' -type f -print0 | while IFS= read -r -d '' enc; do
  i=$((i + 1))
  name=$(basename "$enc" .000)
  echo "PROGRESS: Processing $name ($i/$count)"
  layers=$(ogrinfo -so "$enc" 2>>{err_log} | grep -E '^[0-9]+:' | awk -F': ' '{{print $2}}' | awk '{{print $1}}')
  printf '%s\\n' $layers | xargs -P {parallel} -I '{{}}' sh -c '
    layer="$1" enc="$2" name="$3" multi="$4"
    case "$layer" in {skip}) exit 0 ;; esac
    if [ "$multi" = "1" ]; then outname="${{layer}}_${{name}}"; else outname="$layer"; fi
    if [ "$layer" = "SOUNDG" ]; then
      ogr2ogr -f GeoJSONSeq -makevalid -lco COORDINATE_PRECISION=6 -oo SPLIT_MULTIPOINT=YES -oo ADD_SOUNDG_DEPTH=YES -oo LIST_AS_STRING=YES "{out_dir}/${{outname}}.geojsons" "$enc" "$layer" 2>>{err_log} || true
    else
      ogr2ogr -f GeoJSONSeq -makevalid -lco COORDINATE_PRECISION=6 -oo LIST_AS_STRING=YES "{out_dir}/${{outname}}.geojsons" "$enc" "$layer" 2>>{err_log} || true
    fi
  ' _ '{{}}' "$enc" "$name" "{multi_arg}"
done
echo "PROGRESS: Export complete"
"""

def _build_export_script(multi_file: bool, parallelism: int, in_dir: str, out_dir: str) -> str:
    err_log = f'{out_dir}/{EXPORT_ERROR_FILE_BASENAME}'
    outname = '"${layer}_${name}"' if multi_file else '"${layer}"'
    if max(1, parallelism) == 1:
        return _SEQ_TMPL.format(
            err_log=err_log, in_dir=in_dir, out_dir=out_dir,
            skip=_SKIP_LAYERS, outname=outname,
        )
    return _PAR_TMPL.format(
        err_log=err_log, in_dir=in_dir, out_dir=out_dir,
        skip=_SKIP_LAYERS, parallel=max(1, parallelism),
        multi_arg='1' if multi_file else '0',
    )

# ── GeoJSONSeq streaming consolidation ───────────────────────────────────────

def _iter_geojsonsq(path: str):
    """Yield parsed feature dicts from a GeoJSONSeq file one at a time."""
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line.startswith(_RS):
                line = line[1:]
            if not line:
                continue
            try:
                yield json.loads(line)
            except json.JSONDecodeError:
                continue

def _flatten_list_properties(feature: dict) -> dict:
    """Safety net: join any remaining OGR list attributes into comma strings."""
    props = feature.get('properties')
    if not isinstance(props, dict):
        return feature
    out, changed = {}, False
    for k, v in props.items():
        if isinstance(v, list):
            out[k] = ','.join(str(x) for x in v)
            changed = True
        else:
            out[k] = v
    return {**feature, 'properties': out} if changed else feature

def _with_tippecanoe_minzoom(feature: dict, minzoom: int) -> dict:
    existing = feature.get('tippecanoe') if isinstance(feature.get('tippecanoe'), dict) else {}
    return {**feature, 'tippecanoe': {**existing, 'minzoom': minzoom}}

def _consolidate(geojsons_dir: str, user_minzoom: int) -> list[dict]:
    """
    Stream-merge per-chart-per-layer .geojsons files into one RS-prefixed
    GeoJSONSeq per layer. Reads line-by-line — never loads a whole file.
    RS prefix causes tippecanoe to auto-enable parallel reads.
    """
    files = [f for f in os.listdir(geojsons_dir) if f.endswith('.geojsons')]

    groups: dict[str, list[str]] = {}
    for f in files:
        if os.path.getsize(os.path.join(geojsons_dir, f)) == 0:
            continue
        base = f.removesuffix('.geojsons')
        idx = base.rfind('_')
        tail_has_digit = idx != -1 and any(c.isdigit() for c in base[idx + 1:])
        layer = base[:idx] if tail_has_digit else base
        groups.setdefault(layer, []).append(f)

    merged_dir = os.path.join(geojsons_dir, '.merged')
    os.makedirs(merged_dir, exist_ok=True)

    consolidated = []
    for layer, sources in groups.items():
        out_path = os.path.join(merged_dir, f'{layer}.geojsons')
        with open(out_path, 'w') as out:
            for source in sources:
                band = highest_band_for_files([source])
                band_floor = BAND_MIN_ZOOM.get(band) if band is not None else None
                for feat in _iter_geojsonsq(os.path.join(geojsons_dir, source)):
                    feat = _flatten_list_properties(feat)
                    if props := feat.get('properties'):
                        feat = {**feat, 'properties': {k: v for k, v in props.items() if k not in _SKIP_PROPERTIES}}
                    scamin_floor = scamin_to_minzoom((feat.get('properties') or {}).get('SCAMIN'))
                    floors = [x for x in (band_floor, scamin_floor) if x is not None]
                    if floors:
                        feat = _with_tippecanoe_minzoom(feat, max(user_minzoom, *floors))
                    out.write(f'{_RS}{json.dumps(feat, separators=(",", ":"))}\n')
        consolidated.append({'file': out_path, 'source_files': sources})

    return consolidated

# ── Pipeline steps ────────────────────────────────────────────────────────────

def extract_zip(zip_path: str, target_dir: str) -> list[str]:
    with zipfile.ZipFile(zip_path) as zf:
        zf.extractall(target_dir)
    return [os.path.join(r, f) for r, _, fs in os.walk(target_dir) for f in fs]

def find_enc_files(directory: str) -> list[str]:
    return [
        os.path.join(root, f)
        for root, _, files in os.walk(directory)
        for f in files
        if f.endswith('.000') and not f.startswith('._')
    ]

def _export_layers(enc_dir: str, enc_files: list[str], out_dir: str) -> None:
    script = _build_export_script(len(enc_files) > 1, os.cpu_count() or 1, enc_dir, out_dir)
    rc = subprocess.run(['bash', '-c', script]).returncode
    if rc != 0:
        raise RuntimeError(f'GDAL export failed (exit {rc})')
    if not any(
        os.path.getsize(os.path.join(out_dir, f)) > 0
        for f in os.listdir(out_dir) if f.endswith('.geojsons')
    ):
        err = os.path.join(out_dir, EXPORT_ERROR_FILE_BASENAME)
        try:
            lines = open(err).read().splitlines()
            print('GDAL export errors:\n' + '\n'.join(f'  {l}' for l in lines[:10]))
        except OSError:
            pass
        raise RuntimeError('GDAL export produced no output')

def _run_tippecanoe(
    geojsons_dir: str, output_mbtiles: str,
    minzoom: int, maxzoom: int, threads: int | None = None,
) -> None:
    layers = _consolidate(geojsons_dir, minzoom)
    if not layers:
        raise RuntimeError('No GeoJSONSeq layers to tile')

    merged_dir = os.path.dirname(layers[0]['file'])
    layer_args: list[str] = []
    by_band: dict = {}
    for info in layers:
        name = os.path.basename(info['file']).removesuffix('.geojsons')
        layer_args += ['-L', f'{name}:{info["file"]}']
        b = highest_band_for_files(info['source_files'])
        by_band.setdefault(b, []).append(name)

    for band in sorted(by_band, key=lambda x: x if x is not None else float('inf')):
        names = by_band[band]
        floor = max(minzoom, BAND_MIN_ZOOM.get(band, minzoom)) if band is not None else minzoom
        sample = ', '.join(names[:6]) + (', …' if len(names) > 6 else '')
        print(f'  Band {band or "?"}: {len(names)} layers from z{floor} ({sample})')

    ncpus = threads or (os.cpu_count() or 1)
    rc = subprocess.run(
        [
            'tippecanoe',
            '-o', output_mbtiles,
            '-z', str(maxzoom), '-Z', str(minzoom),
            '--no-tile-size-limit', '--no-feature-limit',
            '--no-simplification-of-shared-nodes',
            '--no-line-simplification',
            '--no-tiny-polygon-reduction',
            '--buffer=80', '--force',
            *layer_args,
        ],
        env={**os.environ, 'TIPPECANOE_MAX_THREADS': str(ncpus)},
    ).returncode
    if rc != 0:
        raise RuntimeError(f'tippecanoe failed (exit {rc})')
    shutil.rmtree(merged_dir, ignore_errors=True)

def _run_tile_join(inputs: list[str], output: str) -> None:
    if len(inputs) == 1:
        try:
            os.rename(inputs[0], output)
        except OSError:
            shutil.copy2(inputs[0], output)
            os.unlink(inputs[0])
        return
    ncpus = os.cpu_count() or 1
    print(f'Joining {len(inputs)} per-band tile sets...')
    rc = subprocess.run(
        ['tile-join', '-o', output, '--no-tile-size-limit', '--force', *inputs],
        env={**os.environ, 'TIPPECANOE_MAX_THREADS': str(ncpus)},
    ).returncode
    if rc != 0:
        raise RuntimeError(f'tile-join failed (exit {rc})')

def _run_band(
    band: int | None,
    cells: list[str],
    label: str,
    enc_dir: str,
    tmp_dir: str,
    user_min: int,
    user_max: int,
    threads: int,
) -> str | None:
    """Full export → consolidate → tippecanoe for one band bucket."""
    if band is not None:
        b_min = max(user_min, BAND_MIN_ZOOM.get(band, user_min))
        b_max = min(user_max, BAND_MAX_ZOOM.get(band, user_max))
        if b_min > b_max:
            print(f'{label} skipped (z{user_max} below band floor z{b_min})')
            return None
        note = []
        if b_min > user_min: note.append(f'raised to z{b_min}')
        if b_max < user_max: note.append(f'clamped to z{b_max}')
        suffix = f' ({", ".join(note)})' if note else ''
        print(f'{label} z{b_min}-z{b_max}{suffix}, {len(cells)} cell(s)')
    else:
        b_min, b_max = user_min, user_max
        print(f'{label} z{b_min}-z{b_max}, {len(cells)} cell(s)')

    slug = str(band) if band is not None else 'unbanded'
    band_enc = os.path.join(tmp_dir, f'enc-{slug}')
    band_out = os.path.join(tmp_dir, f'geojsons-{slug}')
    os.makedirs(band_enc, exist_ok=True)
    os.makedirs(band_out, exist_ok=True)

    for cell in cells:
        rel = os.path.relpath(cell, enc_dir)
        link = os.path.join(band_enc, rel)
        os.makedirs(os.path.dirname(link), exist_ok=True)
        try:
            os.link(cell, link)
        except OSError:
            shutil.copy2(cell, link)

    print(f'{label} Exporting...')
    _export_layers(band_enc, cells, band_out)

    mbtiles = os.path.join(tmp_dir, f'band-{slug}.mbtiles')
    print(f'{label} Tippecanoe z{b_min}-z{b_max} ({threads} threads)')
    _run_tippecanoe(band_out, mbtiles, b_min, b_max, threads)
    return mbtiles

def _per_band_pipeline(
    enc_dir: str, enc_files: list[str],
    tmp_dir: str, options: dict, output: str,
) -> None:
    grouping = group_cells_by_band(enc_files)
    user_min = options.get('minzoom', 9)
    user_max = options.get('maxzoom', 16)
    unbanded = grouping['unbanded']

    total = len(grouping['bands']) + (1 if unbanded else 0)
    # Divide CPU budget across concurrently running tippecanoe instances.
    # Smaller bands finish early, freeing threads for the heaviest band.
    threads = max(1, (os.cpu_count() or 1) // total)

    print(
        f"Per-band pipeline: bands=[{', '.join(str(b) for b in grouping['bands'])}]"
        + (f' + {len(unbanded)} unbanded' if unbanded else '')
        + f', {threads} threads/tippecanoe'
    )

    with ThreadPoolExecutor(max_workers=total) as pool:
        futs = {}
        for i, band in enumerate(grouping['bands'], 1):
            label = f'[band {band} {i}/{total}]'
            futs[pool.submit(
                _run_band, band, grouping['by_band'].get(band, []),
                label, enc_dir, tmp_dir, user_min, user_max, threads,
            )] = label
        if unbanded:
            label = f'[unbanded {len(grouping["bands"]) + 1}/{total}]'
            futs[pool.submit(
                _run_band, None, unbanded,
                label, enc_dir, tmp_dir, user_min, user_max, threads,
            )] = label

        intermediates = []
        for fut in as_completed(futs):
            result = fut.result()
            if result:
                intermediates.append(result)
                print(f'{futs[fut]} → {os.path.basename(result)}')

    if not intermediates:
        raise RuntimeError('All bands skipped — no tile output.')
    _run_tile_join(intermediates, output)

# ── MBTiles metadata ──────────────────────────────────────────────────────────

def patch_mbtiles(path: str, name: str) -> None:
    try:
        conn = sqlite3.connect(path)
        try:
            # tippecanoe omits UNIQUE on metadata(name); add it so INSERT OR REPLACE works.
            conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS _metadata_name ON metadata(name)")
            with conn:
                conn.executemany(
                    "INSERT OR REPLACE INTO metadata(name, value) VALUES (?, ?)",
                    [('type', 'S-57'), ('name', f'S-57 {name}')],
                )
        finally:
            conn.close()
        print(f"MBTiles: type=S-57, name='S-57 {name}'")
    except Exception as e:
        print(f'WARNING: metadata patch failed: {e}', file=sys.stderr)

# ── Main entry point ──────────────────────────────────────────────────────────

def process(input_path: str, output: str, options: dict | None = None) -> None:
    options = options or {}
    tmp = tempfile.mkdtemp(prefix='s57_')
    try:
        geojsons_dir = os.path.join(tmp, 'geojsons')
        os.makedirs(geojsons_dir)

        if os.path.isdir(input_path):
            enc_dir = input_path
            print(f'Using ENC directory: {enc_dir}')
        else:
            enc_dir = os.path.join(tmp, 'enc')
            os.makedirs(enc_dir)
            print('Extracting ENC files...')
            extracted = extract_zip(input_path, enc_dir)
            print(f'Extracted {len(extracted)} files')

        enc_files = find_enc_files(enc_dir)
        if not enc_files:
            raise RuntimeError('No S-57 ENC files (.000) found')
        print(f'Found {len(enc_files)} ENC files')

        grouping = group_cells_by_band(enc_files)
        bucket_count = len(grouping['bands']) + (1 if grouping['unbanded'] else 0)
        user_max = options.get('maxzoom', 16)
        clamp = band_clamped_maxzoom([os.path.basename(f) for f in enc_files], user_max)

        os.makedirs(os.path.dirname(output) or '.', exist_ok=True)

        if bucket_count >= 2:
            bands_str = ', '.join(str(b) for b in grouping['bands']) or '(none)'
            unbanded_str = f' + {len(grouping["unbanded"])} unbanded' if grouping['unbanded'] else ''
            print(f'Per-band pipeline: {bucket_count} buckets [bands {bands_str}{unbanded_str}]')
            _per_band_pipeline(enc_dir, enc_files, tmp, options, output)
        else:
            print(f'Single-pass: {len(enc_files)} ENC file(s)')
            if clamp['highest_band'] and clamp['effective'] < user_max:
                print(f"Maxzoom clamped to z{clamp['effective']} (band {clamp['highest_band']})")
            _export_layers(enc_dir, enc_files, geojsons_dir)
            ncpus = os.cpu_count() or 1
            _run_tippecanoe(geojsons_dir, output, options.get('minzoom', 9), clamp['effective'], ncpus)

        if not os.path.exists(output):
            raise RuntimeError('tippecanoe finished but output not found')

        patch_mbtiles(output, os.path.basename(output).removesuffix('.mbtiles'))
        size_mb = os.path.getsize(output) / (1024 * 1024)
        print(f'Done: {output} ({size_mb:.1f} MB)')

    finally:
        shutil.rmtree(tmp, ignore_errors=True)

# ── CLI ───────────────────────────────────────────────────────────────────────

def main() -> None:
    args = sys.argv[1:]
    if not args:
        print('Usage: python3 convert.py <zip-or-dir> [output-dir] [output-name]', file=sys.stderr)
        sys.exit(1)

    input_path = os.path.realpath(args[0])
    if not os.path.exists(input_path):
        print(f'ERROR: input not found: {input_path}', file=sys.stderr)
        sys.exit(1)

    output_dir = os.path.realpath(args[1]) if len(args) > 1 else os.path.join(os.getcwd(), 'output')
    base = os.path.basename(input_path.rstrip('/'))
    default_name = re.sub(r'-+', '-', re.sub(r'[\s()]', '-', os.path.splitext(base)[0]))
    output_name = args[2] if len(args) > 2 else default_name
    output = os.path.join(output_dir, f'{output_name}.mbtiles')

    print('=== S-57 ENC Converter ===')
    print(f'Input:  {input_path}')
    print(f'Output: {output}')

    start = time.monotonic()
    process(input_path, output)
    elapsed = time.monotonic() - start
    print(f'Wall time: {elapsed:.1f}s ({elapsed / 60:.1f}m)')

if __name__ == '__main__':
    main()
