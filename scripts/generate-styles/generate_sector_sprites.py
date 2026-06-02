#!/usr/bin/env python3
"""
Merge pre-rasterized light-sector icons into the three theme spritesheets.

Reads a sectors JSON file (list of SI image-name strings written by convert.py),
generates a Pillow-rendered icon for each sector × theme, and appends them to the
existing {theme}_simplified.{png,json} and {theme}_simplified@2x.{png,json} files.

Usage:
    python3 generate_sector_sprites.py <sectors.json> <sprites-dir>

Requires: Pillow  (pip install pillow)
"""
from __future__ import annotations

import json
import math
import os
import sys

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("ERROR: Pillow is required:  pip install pillow", file=sys.stderr)
    sys.exit(1)

THEMES = ["day", "dusk", "night"]
_FILL_IDX = {"day": 3, "dusk": 4, "night": 5}
_LINE_IDX = {"day": 6, "dusk": 7, "night": 8}

ICON_SIZE   = 64   # 1× pixels — fits spritesheet limits and renders at a sane screen size
ICON_RADIUS = 26   # arc radius in the 1× coordinate space (80 * 64/200)


# ── Drawing helpers ────────────────────────────────────────────────────────────

def _hex_rgba(h: str, alpha: int = 230) -> tuple[int, int, int, int]:
    h = h.lstrip('#')
    if len(h) == 3:
        h = h[0]*2 + h[1]*2 + h[2]*2
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), alpha)


def _dashed_line(draw: ImageDraw.ImageDraw,
                 x0: float, y0: float, x1: float, y1: float,
                 color: tuple, width: int = 2,
                 dash: float = 4, gap: float = 6) -> None:
    dx, dy = x1 - x0, y1 - y0
    total = math.hypot(dx, dy)
    if total == 0:
        return
    ux, uy = dx / total, dy / total
    pos = 0.0
    while pos < total:
        end = min(pos + dash, total)
        draw.line(
            [(x0 + ux * pos, y0 + uy * pos), (x0 + ux * end, y0 + uy * end)],
            fill=color, width=width,
        )
        pos += dash + gap


def _draw_icon(sectr1: float, sectr2: float,
               fill_hex: str, line_hex: str,
               size: int, radius: float) -> Image.Image:
    img  = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    cx = cy = size / 2

    fill = _hex_rgba(fill_hex)
    line = _hex_rgba(line_hex)

    # S-57 SECTR1/SECTR2 are bearings FROM the navigator TO the light.
    # Add 180° to get the direction outward from the light (matches the JS viewer).
    start = (sectr1 + 180) % 360
    end   = (sectr2 + 180) % 360
    if end <= start:
        end += 360
    span = end - start
    if span < 1.0 or span >= 359.0:
        # Omnidirectional — draw a full circle instead of a sector
        r = radius * size / ICON_SIZE
        draw.ellipse([cx - r, cy - r, cx + r, cy + r], outline=fill, width=4)
        return img

    r = radius * size / ICON_SIZE

    # Arc endpoints (bearing coords: 0 = north, clockwise)
    a1 = math.radians(start)
    a2 = math.radians(start + span)
    x1 = cx + r * math.sin(a1);  y1 = cy - r * math.cos(a1)
    x2 = cx + r * math.sin(a2);  y2 = cy - r * math.cos(a2)

    # PIL arc: 0 = east (right), clockwise.  Convert from bearing: pil = bearing − 90
    pil_start = (start - 90) % 360
    pil_end   = (start + span - 90) % 360
    draw.arc([cx - r, cy - r, cx + r, cy + r], pil_start, pil_end, fill=fill, width=4)

    _dashed_line(draw, cx, cy, x1, y1, line, width=2)
    _dashed_line(draw, cx, cy, x2, y2, line, width=2)

    return img


# ── Spritesheet helpers ────────────────────────────────────────────────────────

# Conservative WebGL max texture dimension (guaranteed on all modern devices).
_MAX_TEXTURE_SIZE = 4096


def _append_icons(png_path: str, json_path: str,
                  icons: list[tuple[str, Image.Image]],
                  pixel_ratio: int) -> None:
    """
    Add icons to the spritesheet in a 2D grid placed below the existing content.
    All icons must be the same size. Stays within _MAX_TEXTURE_SIZE in width.
    """
    if not icons:
        return

    sheet = Image.open(png_path).convert('RGBA')
    index: dict = json.loads(open(json_path).read())

    icon_w, icon_h = icons[0][1].size
    cols = max(1, min(len(icons), _MAX_TEXTURE_SIZE // icon_w))
    rows = math.ceil(len(icons) / cols)

    new_w = max(sheet.width, cols * icon_w)
    new_h = sheet.height + rows * icon_h

    if new_w > _MAX_TEXTURE_SIZE or new_h > _MAX_TEXTURE_SIZE:
        print(
            f"  WARNING: spritesheet will be {new_w}×{new_h}px — "
            f"exceeds {_MAX_TEXTURE_SIZE}px safe limit. "
            f"Consider reducing ICON_SIZE.",
            file=sys.stderr,
        )

    new_sheet = Image.new('RGBA', (new_w, new_h), (0, 0, 0, 0))
    new_sheet.paste(sheet, (0, 0))

    for i, (name, icon) in enumerate(icons):
        col = i % cols
        row = i // cols
        x = col * icon_w
        y = sheet.height + row * icon_h
        new_sheet.paste(icon, (x, y))
        index[name] = {
            'x': x, 'y': y,
            'width': icon_w, 'height': icon_h,
            'pixelRatio': pixel_ratio, 'sdf': False,
        }

    new_sheet.save(png_path)
    with open(json_path, 'w') as f:
        json.dump(index, f, indent=2)


# ── Main ───────────────────────────────────────────────────────────────────────

def generate_sector_sprites(sectors_json: str, sprites_dir: str) -> None:
    si_values: list[str] = json.loads(open(sectors_json).read())
    if not si_values:
        print("No sector SI values — nothing to do.")
        return

    # Parse: sector_{s1}_{s2}_{dayFill}_{duskFill}_{nightFill}_{dayLine}_{duskLine}_{nightLine}_{radius}
    parsed: list[tuple[str, list[str]]] = []
    for si in si_values:
        parts = si.split('_')
        if len(parts) >= 10 and parts[0] == 'sector':
            parsed.append((si, parts))
        else:
            print(f"  skipping unparseable SI value: {si!r}", file=sys.stderr)

    if not parsed:
        print("No parseable SI values.")
        return

    print(f"Generating sector sprites for {len(parsed)} SI value(s) × {len(THEMES)} themes...")

    for theme in THEMES:
        fi = _FILL_IDX[theme]
        li = _LINE_IDX[theme]
        icons_1x: list[tuple[str, Image.Image]] = []
        icons_2x: list[tuple[str, Image.Image]] = []

        for si, parts in parsed:
            try:
                s1, s2   = float(parts[1]), float(parts[2])
                fill_hex = parts[fi]
                line_hex = parts[li]
                radius   = float(parts[9])
            except (ValueError, IndexError):
                continue

            icons_1x.append((si, _draw_icon(s1, s2, fill_hex, line_hex, ICON_SIZE,     radius)))
            icons_2x.append((si, _draw_icon(s1, s2, fill_hex, line_hex, ICON_SIZE * 2, radius * 2)))

        base = os.path.join(sprites_dir, f'{theme}_simplified')
        _append_icons(f'{base}.png',    f'{base}.json',    icons_1x, pixel_ratio=1)
        _append_icons(f'{base}@2x.png', f'{base}@2x.json', icons_2x, pixel_ratio=2)
        print(f"  {theme}: {len(icons_1x)} icon(s) merged")


if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <sectors.json> <sprites-dir>", file=sys.stderr)
        sys.exit(1)
    generate_sector_sprites(sys.argv[1], sys.argv[2])
