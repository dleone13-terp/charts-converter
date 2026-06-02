import maplibregl from 'maplibre-gl';

// ── State ─────────────────────────────────────────────────────────────────────

let map = null;

function onStyleImageMissing(e) {
  if (!map || !e.id) return;
  // Transparent 1×1 placeholder for any sprite not present in the sheet.
  map.addImage(e.id, { width: 1, height: 1, data: new Uint8Array(4) });
}

// ── Style loading ─────────────────────────────────────────────────────────────

function getStyleUrl() {
  return document.getElementById('style-url').value.trim();
}

function getTileSourceUrl() {
  return document.getElementById('tile-source-url').value.trim();
}

function getUseOsm() {
  return document.getElementById('use-osm').checked;
}

async function fetchStyle() {
  const url = getStyleUrl();
  const res = await fetch(url);
  if (!res.ok) throw new Error(`style fetch failed: ${res.status}`);
  const style = await res.json();

  // Override the vector tile source URL (e.g. point at a SignalK chart endpoint
  // instead of the tileserver-gl instance baked into the style).
  const tileSourceUrl = getTileSourceUrl();
  if (tileSourceUrl) {
    for (const src of Object.values(style.sources ?? {})) {
      if (src.type === 'vector') src.url = tileSourceUrl;
    }
  }

  if (getUseOsm()) {
    style.layers = style.layers.filter(l => l.type !== 'background');
    style.sources['osm'] = {
      type: 'raster',
      tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'],
      tileSize: 256,
      attribution: '© <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
    };
    style.layers.unshift({ id: 'osm-base', type: 'raster', source: 'osm' });
  }

  return style;
}

// Read bounds/center from the first vector source's TileJSON URL (embedded in
// the style by tileserver-gl), so the map opens in the right place automatically.
async function initialCamera(style) {
  const vs = Object.values(style.sources ?? {}).find(s => s.type === 'vector' && s.url);
  if (!vs) return { center: [0, 0], zoom: 3 };
  try {
    const res = await fetch(vs.url);
    if (!res.ok) return { center: [0, 0], zoom: 3 };
    const tj = await res.json();
    if (tj.center?.length >= 2) {
      return { center: [tj.center[0], tj.center[1]], zoom: tj.center[2] ?? 6 };
    }
    if (tj.bounds?.length === 4) {
      const [w, s, e, n] = tj.bounds;
      return { center: [(w + e) / 2, (s + n) / 2], zoom: 6 };
    }
  } catch (_) {}
  return { center: [0, 0], zoom: 3 };
}

// ── Map init ──────────────────────────────────────────────────────────────────

async function initMap() {
  setStatus('loading', 'loading…');
  if (map) { map.remove(); map = null; }

  try {
    const style = await fetchStyle();
    const camera = await initialCamera(style);

    map = new maplibregl.Map({
      container: 'map',
      style,
      ...camera,
      attributionControl: false,
    });

    map.addControl(new maplibregl.NavigationControl(), 'top-right');
    map.addControl(new maplibregl.AttributionControl({ compact: true }), 'bottom-right');

    map.on('load', () => setStatus('ok', 'connected'));
    map.on('error', e => setStatus('err', `error: ${e.error?.message ?? 'unknown'}`));
    map.on('click', handleClick);
    map.on('styleimagemissing', onStyleImageMissing);

  } catch (err) {
    setStatus('err', String(err));
  }
}

// ── Style reload (preserves viewport) ────────────────────────────────────────

async function reloadStyle() {
  if (!map) return;
  setStatus('loading', 'switching…');
  try {
    // Purge cached sector SVGs — they encode per-theme colors and must be
    // regenerated after a theme switch.
    map.listImages()
      .filter(id => id.startsWith('sector_'))
      .forEach(id => map.removeImage(id));

    const center = map.getCenter();
    const zoom = map.getZoom();
    const bearing = map.getBearing();
    const pitch = map.getPitch();

    const style = await fetchStyle();
    map.setStyle(style);
    map.once('style.load', () => {
      map.jumpTo({ center, zoom, bearing, pitch });
      setStatus('ok', 'connected');
    });
  } catch (err) {
    setStatus('err', String(err));
  }
}

// ── Feature info ──────────────────────────────────────────────────────────────

function handleClick(e) {
  const features = map.queryRenderedFeatures([
    [e.point.x - 5, e.point.y - 5],
    [e.point.x + 5, e.point.y + 5],
  ]);
  const el = document.getElementById('feature-info');

  if (!features.length) {
    el.innerHTML = '<span class="empty">Click a feature on the map</span>';
    return;
  }

  const f = features[0];
  const props = f.properties ?? {};
  const STYLE_KEYS = ['SY', 'AC', 'LC', 'AP', 'LP', 'EA', 'SI'];
  const rows = [
    `<b>layer:</b> ${f.sourceLayer ?? '—'}`,
    `<b>type:</b> ${f.geometry?.type ?? '—'}`,
  ];
  for (const k of STYLE_KEYS) {
    if (props[k] != null) rows.push(`<b>${k}:</b> ${props[k]}`);
  }
  for (const [k, v] of Object.entries(props)) {
    if (!STYLE_KEYS.includes(k) && k !== 'layer') rows.push(`<b>${k}:</b> ${v}`);
  }
  el.innerHTML = rows.join('<br>');
}

// ── UI wiring ─────────────────────────────────────────────────────────────────

function setStatus(state, text) {
  const el = document.getElementById('status');
  el.textContent = text;
  el.className = state;
}

document.getElementById('style-url').addEventListener('change', initMap);
document.getElementById('tile-source-url').addEventListener('change', initMap);
document.getElementById('use-osm').addEventListener('change', reloadStyle);
document.getElementById('load-btn').addEventListener('click', initMap);

// ── Boot ──────────────────────────────────────────────────────────────────────

initMap();
