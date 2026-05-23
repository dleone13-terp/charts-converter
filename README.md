# charts-converter

Automated weekly conversion of [NOAA Electronic Navigational Charts](https://charts.noaa.gov/ENCs/ENCs.shtml) (S-57 format) to MBTiles vector tiles, stored as GitHub Release assets.

## How it works

A GitHub Actions workflow runs every Monday at 06:00 UTC:

1. **Check** — HEAD-requests each NOAA Coast Guard District ZIP and compares `Last-Modified` against `state.json`. Only districts whose ZIP has changed since the last run are queued.
2. **Convert** — One runner per changed district (up to 4 in parallel). Downloads the district ZIP, then uses [`ghcr.io/dirkwa/signalk-charts-provider-simple/charts-toolbox`](https://github.com/dirkwa/signalk-charts-provider-simple) (GDAL + tippecanoe) to convert all S-57 cells to a single merged MBTiles file.
3. **Publish** — Uploads the updated `.mbtiles` files to the [`noaa-latest` release](../../releases/tag/noaa-latest), regenerates `manifest.json`, and commits the updated `state.json`.

## Output

The `noaa-latest` release contains one file per Coast Guard District:

| File | Coverage |
|---|---|
| `noaa-enc-01CGD.mbtiles` | 1st District — New England |
| `noaa-enc-05CGD.mbtiles` | 5th District — Mid-Atlantic |
| `noaa-enc-07CGD.mbtiles` | 7th District — Southeast |
| `noaa-enc-08CGD.mbtiles` | 8th District — Gulf Coast |
| `noaa-enc-09CGD.mbtiles` | 9th District — Great Lakes |
| `noaa-enc-11CGD.mbtiles` | 11th District — Pacific Southwest |
| `noaa-enc-13CGD.mbtiles` | 13th District — Pacific Northwest |
| `noaa-enc-14CGD.mbtiles` | 14th District — Hawaii / Pacific Islands |
| `noaa-enc-17CGD.mbtiles` | 17th District — Alaska |
| `manifest.json` | Machine-readable index with download URLs and conversion dates |

Tile zoom range: z4–z16. Format: vector tiles (MVT / pbf).

## Incremental updates

`state.json` records the HTTP `Last-Modified` timestamp for each district ZIP as last seen. On each weekly run only districts whose ZIP has changed are reconverted — typically 1–3 districts. If a conversion fails the district is retried on the next run.

## Manual trigger

Go to **Actions → Convert NOAA ENCs to MBTiles → Run workflow**. Leave the input blank for auto-detect, or enter district IDs (e.g. `09CGD,17CGD`) to force specific districts, or `all` to reconvert everything.

## Using with Signal K

Copy (or symlink) a `.mbtiles` file into your Signal K chart path. The [`signalk-charts-provider-simple`](https://github.com/dirkwa/signalk-charts-provider-simple) plugin will pick it up automatically.

The `manifest.json` URL can also be registered as a catalog source in the plugin's Chart Catalog tab for one-click downloads directly from the boat.
