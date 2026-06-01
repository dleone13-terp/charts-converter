/**
 * ENC Chart Viewer — OpenLayers + s57style WASM
 *
 * Three style modes:
 *   "wasm-live"  — WASM engine computes style from raw S-57 attributes each
 *                  call (with an LRU cache keyed on relevant props).
 *   "prebaked"   — reads SY/AC/LC properties already stamped by convert.py
 *                  and resolves hex colors with a static inline lookup.
 *   "none"       — no style function; useful as a rendering baseline.
 */

import Map from 'ol/Map';
import View from 'ol/View';
import VectorTileLayer from 'ol/layer/VectorTile';
import VectorTileSource from 'ol/source/VectorTile';
import MVT from 'ol/format/MVT';
import { fromLonLat } from 'ol/proj';
import Style from 'ol/style/Style';
import Fill from 'ol/style/Fill';
import Stroke from 'ol/style/Stroke';
import CircleStyle from 'ol/style/Circle';
import Text from 'ol/style/Text';

// WASM module — built with:  npm run wasm  (wasm-pack --target bundler)
import { WasmStyleEngine, create_engine } from './wasm-pkg/s57style_wasm.js';

// ── State ──────────────────────────────────────────────────────────────────

let engine = null;
let styleMode = 'wasm-live'; // 'wasm-live' | 'prebaked' | 'none'
let vtLayer = null;
let olMap = null;

const config = {
  shallow_depth: 3.0,
  safety_depth: 6.0,
  deep_depth: 9.0,
  theme: 'Day',
  depth_unit: 'Meters',
};

// ── LRU style cache (avoids recomputing style for identical feature props) ──

const CACHE_SIZE = 8000;
const styleCache = new Map();

function cacheGet(key) {
  if (!styleCache.has(key)) return undefined;
  // Move to end (most-recently-used)
  const val = styleCache.get(key);
  styleCache.delete(key);
  styleCache.set(key, val);
  return val;
}

function cacheSet(key, val) {
  if (styleCache.size >= CACHE_SIZE) {
    // Evict least-recently-used (first entry)
    styleCache.delete(styleCache.keys().next().value);
  }
  styleCache.set(key, val);
}

// ── Performance tracking ────────────────────────────────────────────────────

const perf = {
  calls: 0,          // WASM engine calls (cache misses)
  cacheHits: 0,
  totalMs: 0,        // cumulative WASM time (ms)
  renderFrames: 0,
  startTime: performance.now(),
  lastFrameMs: 0,
};

// ── S57 color table (inline, for "pre-baked" mode; matches colors.json DAY) ─

const S57_COLORS_DAY = {
  NODTA:'#93AEBB',CURSR:'#E38039',CHBLK:'#000000',CHGRD:'#4C5B63',
  CHGRF:'#768C97',CHRED:'#EA5471',CHGRN:'#52E83B',CHYLW:'#E1E139',
  CHMGD:'#C045D1',CHMGF:'#CBA9F9',CHBRN:'#A19653',CHWHT:'#C9EDFF',
  LITRD:'#EA5471',LITGN:'#52E83B',LITYW:'#E1E139',ISDNG:'#C045D1',
  DNGHL:'#EA5471',TRFCD:'#C045D1',TRFCF:'#CBA9F9',LANDA:'#BFBE8F',
  LANDF:'#8D642E',CSTLN:'#4C5B63',SNDG1:'#768C97',SNDG2:'#000000',
  DEPDW:'#C9EDFF',DEPMD:'#A7D9FB',DEPMS:'#82CAFF',DEPVS:'#61B7FF',
  DEPIT:'#58AF9C',RADHI:'#52E83B',RESBL:'#2E7BFF',
};
const S57_COLORS_DUSK = {
  DEPDW:'#000000',DEPMD:'#0F1B21',DEPMS:'#1D3246',DEPVS:'#1E4165',
  DEPIT:'#234C44',LANDA:'#40402E',LANDF:'#7F5A29',CHBLK:'#6B7F89',
  CHGRD:'#6B7F89',CSTLN:'#6B7F89',LITRD:'#9B3549',LITGN:'#2F8E20',
  LITYW:'#8B8B1F',TRFCD:'#826CA1',TRFCF:'#772782',CHMGD:'#826CA1',
  CHBRN:'#57502A',
};
const S57_COLORS_NIGHT = {
  DEPDW:'#000000',DEPMD:'#03070A',DEPMS:'#050E16',DEPVS:'#071727',
  DEPIT:'#0B201C',LANDA:'#17160E',LANDF:'#2F1F0A',CHBLK:'#252D31',
  CHGRD:'#252D31',CSTLN:'#252D31',LITRD:'#390E16',LITGN:'#0C3406',
  LITYW:'#323206',TRFCD:'#411247',TRFCF:'#411247',CHMGD:'#411247',
  CHBRN:'#211E0C',
};

function s57ColorHex(name) {
  if (!name) return null;
  const table = config.theme === 'Night' ? S57_COLORS_NIGHT
              : config.theme === 'Dusk'  ? S57_COLORS_DUSK
              : S57_COLORS_DAY;
  return (table[name] ?? S57_COLORS_DAY[name]) || null;
}

// ── WASM engine helpers ─────────────────────────────────────────────────────

function buildEngine() {
  try {
    if (engine) engine.free?.();
    engine = create_engine(
      config.shallow_depth,
      config.safety_depth,
      config.deep_depth,
      config.theme,
      config.depth_unit,
    );
    styleCache.clear();
    setWasmStatus('ok', '✓ WASM ready');
  } catch (e) {
    setWasmStatus('err', '✗ WASM error');
    console.error('WASM engine error:', e);
  }
}

// ── OpenLayers style functions ──────────────────────────────────────────────

/**
 * "wasm-live" mode: call engine.style_feature_resolved for every unique
 * combination of relevant properties (cached).
 */
function styleWasmLive(feature, _resolution) {
  if (!engine) return null;

  const props = feature.getProperties();
  const layerName = props.layer || '';
  if (!layerName) return null;

  // Cache key: for most layers only a handful of attrs determine the style
  const cacheKey = `${config.theme}|${layerName}|${styleCacheKey(layerName, props)}`;

  let resolved = cacheGet(cacheKey);
  if (resolved === undefined) {
    const t0 = performance.now();
    try {
      resolved = JSON.parse(engine.style_feature_resolved(layerName, JSON.stringify(props)));
    } catch {
      resolved = {};
    }
    const dt = performance.now() - t0;
    perf.calls++;
    perf.totalMs += dt;
    cacheSet(cacheKey, resolved);
  } else {
    perf.cacheHits++;
  }

  return buildOlStyle(resolved, layerName, feature);
}

/**
 * "prebaked" mode: read SY/AC/LC from tile properties (stamped by convert.py)
 * and resolve hex colors inline, no WASM call.
 */
function stylePreBaked(feature, _resolution) {
  const props = feature.getProperties();
  const layerName = props.layer || '';

  const resolved = {
    symbol: props.SY || null,
    area_color: s57ColorHex(props.AC),
    line_color: s57ColorHex(props.LC),
    area_pattern: props.AP || null,
    exclude_area_point_symbol: !!props.EA,
    extra: {},
  };

  // Carry over pre-computed depth fields for soundings
  if (props.METERS_W !== undefined) {
    resolved.extra.METERS_W = props.METERS_W;
    resolved.extra.METERS_T = props.METERS_T;
  }

  return buildOlStyle(resolved, layerName, feature);
}

/**
 * Build an OL Style from a ResolvedFeatureStyle.
 */
function buildOlStyle(s, layerName, feature) {
  // Soundings — show depth as text
  if (layerName === 'SOUNDG') {
    const mw = s.extra?.METERS_W ?? feature.get('METERS_W');
    const mt = s.extra?.METERS_T ?? feature.get('METERS_T');
    if (mw != null) {
      const label = (mt > 0) ? `${mw}·${mt}` : `${mw}`;
      return new Style({
        text: new Text({
          text: label,
          font: 'bold 10px "Courier New", monospace',
          fill: new Fill({ color: s.area_color || '#000' }),
          stroke: new Stroke({ color: s.line_color || '#C9EDFF', width: 2 }),
          textAlign: 'center',
        }),
      });
    }
  }

  const geomType = geomTypeName(feature);
  const fillColor = s.area_color ? hexAlpha(s.area_color, 0.80) : null;
  const lineColor = s.line_color || null;

  if (geomType === 'Polygon') {
    return new Style({
      fill: fillColor ? new Fill({ color: fillColor }) : undefined,
      stroke: lineColor ? new Stroke({ color: lineColor, width: 1 }) : undefined,
    });
  }

  if (geomType === 'LineString') {
    return new Style({
      stroke: lineColor
        ? new Stroke({ color: lineColor, width: 1.5 })
        : new Stroke({ color: '#4C5B63', width: 1 }),
    });
  }

  if (geomType === 'Point') {
    const color = s.area_color || s.line_color || '#888';
    return new Style({
      image: new CircleStyle({
        radius: 4,
        fill: new Fill({ color: hexAlpha(color, 0.9) }),
        stroke: new Stroke({ color: '#000', width: 1 }),
      }),
    });
  }

  return null;
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/** Minimal props that determine style for common layer types. */
function styleCacheKey(layerName, props) {
  // For most layers, a few key attrs decide the style
  const relevant = [
    props.DRVAL1, props.DRVAL2,           // DEPARE
    props.CATCAM, props.CATCAM,           // BCNCAR/BOYCAR
    props.CATLAM, props.BCNSHP,           // BCNLAT
    props.COLOUR,                          // LIGHTS
    props.CATWRK, props.WATLEV, props.VALSOU, // WRECKS/OBSTRN
    props.TOPSHP, props['_PTFM'],         // TOPMAR
    props.CATOBS, props.QUASOU,           // OBSTRN
    // For simple layers the layer name alone is enough, but hashing extras
    // avoids stale hits for layers this list doesn't enumerate.
  ];
  return relevant.filter(v => v != null).join('|');
}

function geomTypeName(feature) {
  // RenderFeature (VectorTile) uses feature.type_ (ol internal)
  const t = feature.getGeometry?.()?.getType?.() ?? feature.type_;
  if (!t) return 'Unknown';
  if (t.includes('Polygon'))    return 'Polygon';
  if (t.includes('LineString')) return 'LineString';
  if (t.includes('Point'))      return 'Point';
  return t;
}

function hexAlpha(hex, a) {
  if (!hex?.startsWith('#')) return hex;
  const r = parseInt(hex.slice(1,3),16);
  const g = parseInt(hex.slice(3,5),16);
  const b = parseInt(hex.slice(5,7),16);
  return `rgba(${r},${g},${b},${a})`;
}

// ── Map setup ───────────────────────────────────────────────────────────────

function currentStyleFn() {
  if (styleMode === 'wasm-live') return styleWasmLive;
  if (styleMode === 'prebaked')  return stylePreBaked;
  return null;
}

function buildTileUrl(base, layerId) {
  base = base.replace(/\/$/, '');
  // martin format: /<layer>/{z}/{x}/{y}
  // tileserver-gl: /data/<id>/{z}/{x}/{y}.pbf
  // generic: /{z}/{x}/{y}
  if (layerId && !base.includes('{z}')) {
    return `${base}/${layerId}/{z}/{x}/{y}`;
  }
  return base.includes('{z}') ? base : `${base}/{z}/{x}/{y}`;
}

function connectTiles() {
  const tileUrl = buildTileUrl(
    document.getElementById('tile-url').value.trim(),
    document.getElementById('layer-id').value.trim(),
  );

  if (vtLayer) olMap.removeLayer(vtLayer);

  vtLayer = new VectorTileLayer({
    source: new VectorTileSource({
      format: new MVT(),
      url: tileUrl,
      maxZoom: 18,
    }),
    style: currentStyleFn(),
    renderMode: 'hybrid', // polygons + vector lines/points
  });

  olMap.addLayer(vtLayer);

  // Performance: count rendered frames
  vtLayer.on('postrender', (e) => {
    perf.renderFrames++;
    perf.lastFrameMs = e.frameState?.time ?? performance.now();
  });
}

function applyMode(mode) {
  styleMode = mode;
  document.querySelectorAll('.mode-btn').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.mode === mode);
  });
  if (vtLayer) {
    vtLayer.setStyle(currentStyleFn());
    styleCache.clear();
    vtLayer.changed();
  }
}

// ── Click → feature info ────────────────────────────────────────────────────

function showFeatureInfo(feature) {
  const props = feature.getProperties();
  const layer = props.layer || '(unknown)';
  const lines = [`<b>Layer:</b> ${layer}`];

  // Show style-relevant properties first
  const styleKeys = ['SY','AC','LC','AP','LP','EA','METERS','METERS_W','METERS_T',
                     'DRVAL1','DRVAL2','CATCAM','CATLAM','COLOUR','WATLEV','VALSOU','CATWRK'];
  styleKeys.forEach(k => {
    if (props[k] != null) lines.push(`<b>${k}:</b> ${props[k]}`);
  });

  // Show all other props
  Object.entries(props).forEach(([k, v]) => {
    if (!styleKeys.includes(k) && k !== 'layer' && v != null) {
      lines.push(`${k}: ${v}`);
    }
  });

  document.getElementById('feature-info').innerHTML = lines.join('<br>');
}

// ── Performance display ─────────────────────────────────────────────────────

function renderPerfStats() {
  const elapsed = (performance.now() - perf.startTime) / 1000;
  const total = perf.calls + perf.cacheHits;
  const cps  = elapsed > 0 ? (total / elapsed).toFixed(0) : '—';
  const avg  = perf.calls > 0 ? (perf.totalMs / perf.calls).toFixed(3) : '—';
  const hit  = total > 0 ? ((perf.cacheHits / total) * 100).toFixed(0) : '—';

  const avgNum = parseFloat(avg);
  const avgClass = isNaN(avgNum) ? '' : avgNum < 0.05 ? 'fast' : avgNum > 0.5 ? 'slow' : '';

  const rows = [
    ['Mode',          styleMode],
    ['Total calls',   total.toLocaleString()],
    ['Cache hits',    `${perf.cacheHits.toLocaleString()} (${hit}%)`],
    ['WASM calls',    perf.calls.toLocaleString()],
    [`Avg WASM`,      `<span class="perf-val ${avgClass}">${avg} ms</span>`],
    ['WASM total',    `${perf.totalMs.toFixed(1)} ms`],
    ['Rate',          `${cps} calls/s`],
    ['Cache size',    styleCache.size.toLocaleString()],
    ['Render frames', perf.renderFrames.toLocaleString()],
  ];

  document.getElementById('perf-rows').innerHTML = rows.map(([k, v]) =>
    `<div class="perf-row"><span class="perf-key">${k}</span><span class="perf-val">${v}</span></div>`
  ).join('');

  requestAnimationFrame(renderPerfStats);
}

// ── UI wiring ───────────────────────────────────────────────────────────────

function setupUI() {
  // Connect button
  document.getElementById('connect-btn').addEventListener('click', connectTiles);

  // Theme
  document.getElementById('theme').addEventListener('change', e => {
    config.theme = e.target.value;
    buildEngine();
    if (vtLayer) { styleCache.clear(); vtLayer.changed(); }
  });

  // Depth unit
  document.getElementById('depth-unit').addEventListener('change', e => {
    config.depth_unit = e.target.value;
    buildEngine();
    if (vtLayer) { styleCache.clear(); vtLayer.changed(); }
  });

  // Depth sliders
  [
    ['shallow', 'shallow_depth', v => `${v}m`],
    ['safety',  'safety_depth',  v => `${v}m`],
    ['deep',    'deep_depth',    v => `${v}m`],
  ].forEach(([id, cfgKey, fmt]) => {
    const el = document.getElementById(id);
    const lbl = document.getElementById(`${id}-val`);
    el.addEventListener('input', () => {
      const v = parseFloat(el.value);
      lbl.textContent = fmt(v);
      config[cfgKey] = v;
      buildEngine();
      if (vtLayer) { styleCache.clear(); vtLayer.changed(); }
    });
  });

  // Mode buttons
  document.querySelectorAll('.mode-btn').forEach(btn => {
    btn.addEventListener('click', () => applyMode(btn.dataset.mode));
  });
}

function setWasmStatus(cls, text) {
  const el = document.getElementById('wasm-status');
  el.className = cls;
  el.textContent = text;
}

// ── Entry point ─────────────────────────────────────────────────────────────

async function main() {
  setWasmStatus('loading', 'WASM loading…');

  // Initialise map
  olMap = new Map({
    target: 'map',
    layers: [],
    view: new View({
      center: fromLonLat([-70.0, 42.5]), // North-east US coast as default
      zoom: 9,
    }),
  });

  // Click → feature info
  olMap.on('singleclick', evt => {
    olMap.forEachFeatureAtPixel(evt.pixel, (feature) => {
      showFeatureInfo(feature);
      return true; // stop at first
    });
  });

  // Connect tile source if URL params suggest it
  const params = new URLSearchParams(location.search);
  const tileParam = params.get('tiles');
  if (tileParam) {
    document.getElementById('tile-url').value = tileParam;
  }
  const layerParam = params.get('layer');
  if (layerParam) {
    document.getElementById('layer-id').value = layerParam;
  }

  // Build WASM engine
  try {
    buildEngine();
  } catch (e) {
    setWasmStatus('err', '✗ WASM init failed');
    console.error(e);
  }

  // Auto-connect if URL params provided
  if (tileParam) connectTiles();

  setupUI();
  renderPerfStats();
}

main().catch(err => {
  setWasmStatus('err', '✗ startup error');
  console.error(err);
});
