#!/usr/bin/env python3
"""
Generate 9 Mapbox GL style.json files (3 themes × 3 depth units) and copy the
njord spritesheets + Roboto PBF fonts into the serve/ directory.

Usage:
    python3 generate_styles.py [--server URL] [--data-name NAME] [--out-dir DIR]

    --server   Base URL of the tileserver-gl instance  (default: http://localhost:8080)
    --data-name  MBTiles dataset name inside tileserver-gl  (default: de-encs-styled)
    --out-dir    serve/ directory to populate  (default: ../../serve)
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import sys

THEMES = ["Day", "Dusk", "Night"]
DEPTH_UNITS = ["Meters", "Fathoms", "Feet"]

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
CHARTS_DIR = os.path.realpath(os.path.join(SCRIPT_DIR, "../.."))
NJORD_DIR = os.path.realpath(os.path.join(CHARTS_DIR, "../njord"))
NJORD_SPRITES = os.path.join(NJORD_DIR, "server/src/nativeMain/resources/www/sprites")
NJORD_FONTS = os.path.join(NJORD_DIR, "server/src/nativeMain/resources/www/fonts")


def style_name(theme: str, depth_unit: str) -> str:
    return f"{theme.lower()}_{depth_unit.lower()}"


def generate_styles(server: str, data_name: str, out_dir: str) -> None:
    styles_dir = os.path.join(out_dir, "styles")
    sprites_dir = os.path.join(out_dir, "sprites")
    fonts_dir = os.path.join(out_dir, "fonts")
    data_dir = os.path.join(out_dir, "data")

    for d in (styles_dir, sprites_dir, fonts_dir, data_dir):
        os.makedirs(d, exist_ok=True)

    try:
        import s57style as _mod
    except ImportError:
        print("ERROR: s57style module not installed.", file=sys.stderr)
        print("  Build it: cd <openenc-styling-rust>/crates/s57style-python && maturin build --release", file=sys.stderr)
        print("  Then:     pip install <wheel>", file=sys.stderr)
        sys.exit(1)

    source_url = f"{server}/data/{data_name}.json"
    glyphs_url = f"{server}/fonts/{{fontstack}}/{{range}}.pbf"

    print(f"Generating {len(THEMES) * len(DEPTH_UNITS)} style.json files → {styles_dir}")

    for theme in THEMES:
        sprite_url = f"{server}/sprites/{theme.lower()}_simplified"
        for depth_unit in DEPTH_UNITS:
            name = style_name(theme, depth_unit)
            engine = _mod.StyleEngine(theme=theme, depth_unit=depth_unit)
            style_json = engine.mapbox_style_json(source_url, sprite_url, glyphs_url)

            # Patch the style name inside the JSON
            style_obj = json.loads(style_json)
            style_obj["name"] = name
            out_path = os.path.join(styles_dir, f"{name}.json")
            with open(out_path, "w") as f:
                json.dump(style_obj, f, indent=2)
            print(f"  {name}.json")

    # Copy spritesheets from njord
    if os.path.isdir(NJORD_SPRITES):
        sprite_files = [f for f in os.listdir(NJORD_SPRITES) if "simplified" in f]
        print(f"\nCopying {len(sprite_files)} sprite files from njord → {sprites_dir}")
        for fname in sorted(sprite_files):
            src = os.path.join(NJORD_SPRITES, fname)
            dst = os.path.join(sprites_dir, fname)
            shutil.copy2(src, dst)
            print(f"  {fname}")
    else:
        print(f"WARNING: njord sprites not found at {NJORD_SPRITES}", file=sys.stderr)
        print("  You'll need to generate sprites manually or copy them.", file=sys.stderr)

    # Copy fonts from njord
    if os.path.isdir(NJORD_FONTS):
        font_families = os.listdir(NJORD_FONTS)
        print(f"\nCopying {len(font_families)} font families from njord → {fonts_dir}")
        for family in sorted(font_families):
            src = os.path.join(NJORD_FONTS, family)
            dst = os.path.join(fonts_dir, family)
            if os.path.isdir(src):
                if os.path.exists(dst):
                    shutil.rmtree(dst)
                shutil.copytree(src, dst)
                pbf_count = len([f for f in os.listdir(dst) if f.endswith(".pbf")])
                print(f"  {family}/ ({pbf_count} glyphs)")
    else:
        print(f"WARNING: njord fonts not found at {NJORD_FONTS}", file=sys.stderr)

    # Write tileserver-gl config.json
    styles_conf = {}
    for theme in THEMES:
        for depth_unit in DEPTH_UNITS:
            name = style_name(theme, depth_unit)
            styles_conf[name] = {
                "style": f"{name}.json",
                "serve_rendered": False,
            }

    config = {
        "options": {
            "paths": {
                "root": "",
                "fonts": "fonts",
                "sprites": "sprites",
                "styles": "styles",
                "mbtiles": "data",
            }
        },
        "styles": styles_conf,
        "data": {
            data_name: {
                "mbtiles": f"{data_name}.mbtiles"
            }
        },
    }

    config_path = os.path.join(out_dir, "config.json")
    with open(config_path, "w") as f:
        json.dump(config, f, indent=2)
    print(f"\nWrote tileserver-gl config → {config_path}")

    # Print run command
    print(f"""
To serve:
  docker run --rm -it \\
    -v {os.path.abspath(out_dir)}:/data \\
    -p 8080:8080 \\
    maptiler/tileserver-gl --config /data/config.json

Then open the viewer at http://localhost:5173
""")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--server", default="http://localhost:8080",
                        help="Base URL of the tileserver-gl instance")
    parser.add_argument("--data-name", default="de-encs-styled",
                        help="MBTiles dataset name in tileserver-gl")
    parser.add_argument("--out-dir", default=os.path.join(CHARTS_DIR, "serve"),
                        help="serve/ directory to populate")
    args = parser.parse_args()

    generate_styles(args.server, args.data_name, args.out_dir)
