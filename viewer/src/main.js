import maplibregl from 'maplibre-gl';

// ── State ─────────────────────────────────────────────────────────────────────

let theme = 'day';
let depthUnit = 'meters';
let map = null;

function server() {
  return document.getElementById('server-url').value.replace(/\/$/, '');
}
function dataName() {
  return document.getElementById('data-name').value.trim();
}
function styleUrl() {
  return `${server()}/styles/${theme}_${depthUnit}/style.json`;
}

// ── Style fetch + OSM injection ───────────────────────────────────────────────

async function fetchStyle() {
  const res = await fetch(styleUrl());
  if (!res.ok) throw new Error(`style fetch failed: ${res.status}`);
  const style = await res.json();

  // Remove the solid background layer — OSM takes its place as the base.
  style.layers = style.layers.filter(l => l.type !== 'background');

  style.sources['osm'] = {
    type: 'raster',
    tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'],
    tileSize: 256,
    attribution: '© <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
  };

  style.layers.unshift({
    id: 'osm-base',
    type: 'raster',
    source: 'osm',
    paint: { 'raster-opacity': 1.0 },
  });

  return style;
}

async function fetchTileJson() {
  try {
    const res = await fetch(`${server()}/data/${dataName()}.json`);
    if (res.ok) return await res.json();
  } catch (_) {}
  return null;
}

// ── Map init ──────────────────────────────────────────────────────────────────

async function initMap() {
  setStatus('loading', 'loading…');

  if (map) {
    map.remove();
    map = null;
  }

  const [style, tileJson] = await Promise.all([fetchStyle(), fetchTileJson()]);

  let initialCamera = { center: [0, 0], zoom: 2 };
  if (tileJson?.bounds?.length === 4) {
    const [w, s, e, n] = tileJson.bounds;
    initialCamera = { center: [(w + e) / 2, (s + n) / 2], zoom: 6 };
  }

  try {
    map = new maplibregl.Map({
      container: 'map',
      style,
      ...initialCamera,
      attributionControl: false,
    });

    map.addControl(new maplibregl.NavigationControl(), 'top-right');
    map.addControl(new maplibregl.AttributionControl({ compact: true }), 'bottom-right');

    map.on('load', () => setStatus('ok', 'connected'));
    map.on('error', e => setStatus('err', `error: ${e.error?.message ?? 'unknown'}`));
    map.on('click', handleClick);

  } catch (err) {
    setStatus('err', String(err));
  }
}

// ── Style switching ───────────────────────────────────────────────────────────

async function applyStyle() {
  if (!map) return;
  setStatus('loading', 'switching…');
  try {
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
    el.innerHTML = '<span class="empty">No features here</span>';
    return;
  }

  const f = features[0];
  const props = f.properties || {};
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

document.querySelectorAll('[data-theme]').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('[data-theme]').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    theme = btn.dataset.theme;
    applyStyle();
  });
});

document.querySelectorAll('[data-unit]').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('[data-unit]').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    depthUnit = btn.dataset.unit;
    applyStyle();
  });
});

document.getElementById('server-url').addEventListener('change', initMap);
document.getElementById('data-name').addEventListener('change', initMap);

// ── Boot ──────────────────────────────────────────────────────────────────────

initMap();
