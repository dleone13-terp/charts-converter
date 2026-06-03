#!/usr/bin/env python3
"""
Merge light-sector icons into the three theme spritesheets.

Icon naming: sector_{SECTR1}_{SECTR2}_{COLOUR_CODE}
  e.g. sector_270_90_3  (red sector from 270° to 90°)

Colour codes match S-57 COLOUR attribute values:
  1 = white (theme-varying)   3 = red   4 = green   6 = yellow

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

# S-57 colour code → (day_fill, dusk_fill, night_fill)
_FILLS: dict[str, tuple[str, str, str]] = {
    '1': ('#FFFFFF', '#FECC86', '#CC8E3B'),   # white  — theme-varying per S-52
    '3': ('#FF0000', '#FF0000', '#CC0000'),   # red
    '4': ('#00B049', '#00B049', '#007030'),   # green
    '6': ('#FFFF00', '#FFFF00', '#CCCC00'),   # yellow
}
_FILL_DEFAULT = ('#888888', '#888888', '#666666')
_LINE_COLOUR  = '#333333'   # radial line colour (same for all themes)

ICON_SIZE   = 64   # 1× pixels
ICON_RADIUS = 26   # arc radius in 1× coordinate space

_MAX_TEXTURE_SIZE = 4096


# ── Drawing helpers ────────────────────────────────────────────────────────────

def _hex_rgba(h: str, alpha: int = 230) -> tuple[int, int, int, int]:
    h = h.lstrip('#')
    if len(h) == 3:
        h = h[0]*2 + h[1]*2 + h[2]*2
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), alpha)


def _dashed_line(draw: ImageDraw.ImageDraw,
                 x0: float, y0: float, x1: float, y1: float,
                 color: tuple, width: int = 1,
                 dash: float = 3, gap: float = 4) -> None:
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

    # S-57 SECTR1/SECTR2 are bearings FROM navigator TO light.
    # Add 180° to get direction outward from the light.
    start = (sectr1 + 180) % 360
    end   = (sectr2 + 180) % 360
    if end <= start:
        end += 360
    span = end - start

    r = radius * size / ICON_SIZE

    if span < 1.0 or span >= 359.0:
        draw.ellipse([cx - r, cy - r, cx + r, cy + r], outline=fill, width=2)
        return img

    a1 = math.radians(start)
    a2 = math.radians(start + span)
    x1 = cx + r * math.sin(a1);  y1 = cy - r * math.cos(a1)
    x2 = cx + r * math.sin(a2);  y2 = cy - r * math.cos(a2)

    # PIL arc: 0=east, clockwise; convert from bearing: pil = bearing − 90
    pil_start = (start - 90) % 360
    pil_end   = (start + span - 90) % 360
    draw.arc([cx - r, cy - r, cx + r, cy + r], pil_start, pil_end, fill=fill, width=2)

    _dashed_line(draw, cx, cy, x1, y1, line, width=1)
    _dashed_line(draw, cx, cy, x2, y2, line, width=1)

    return img


# ── Spritesheet helpers ────────────────────────────────────────────────────────

def _append_icons(png_path: str, json_path: str,
                  icons: list[tuple[str, Image.Image]],
                  pixel_ratio: int) -> None:
    """Add icons in a 2-D grid placed below the existing sheet content."""
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
            f"exceeds {_MAX_TEXTURE_SIZE}px limit. Reduce ICON_SIZE.",
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
        print("No sector values — nothing to do.")
        return

    # Parse: sector_{sectr1}_{sectr2}_{colour_code}
    parsed: list[tuple[str, float, float, str]] = []
    for si in si_values:
        parts = si.split('_')
        if len(parts) == 4 and parts[0] == 'sector':
            try:
                parsed.append((si, float(parts[1]), float(parts[2]), parts[3]))
            except ValueError:
                print(f"  skipping unparseable: {si!r}", file=sys.stderr)
        else:
            print(f"  skipping unparseable: {si!r}", file=sys.stderr)

    if not parsed:
        print("No parseable sector values.")
        return

    print(f"Generating sector sprites: {len(parsed)} unique configs × {len(THEMES)} themes")

    for t_idx, theme in enumerate(THEMES):
        icons_1x: list[tuple[str, Image.Image]] = []
        icons_2x: list[tuple[str, Image.Image]] = []

        for si, s1, s2, colour in parsed:
            fills = _FILLS.get(colour, _FILL_DEFAULT)
            fill  = fills[t_idx]
            icons_1x.append((si, _draw_icon(s1, s2, fill, _LINE_COLOUR, ICON_SIZE,     ICON_RADIUS)))
            icons_2x.append((si, _draw_icon(s1, s2, fill, _LINE_COLOUR, ICON_SIZE * 2, ICON_RADIUS * 2)))

        base = os.path.join(sprites_dir, f'{theme}_simplified')
        _append_icons(f'{base}.png',    f'{base}.json',    icons_1x, pixel_ratio=1)
        _append_icons(f'{base}@2x.png', f'{base}@2x.json', icons_2x, pixel_ratio=2)
        print(f"  {theme}: {len(icons_1x)} icon(s) merged")


if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <sectors.json> <sprites-dir>", file=sys.stderr)
        sys.exit(1)
    generate_sector_sprites(sys.argv[1], sys.argv[2])
