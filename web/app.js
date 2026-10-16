// Harness for the WASM field compass: UI state, input handling and Canvas 2D drawing.
// The Rust/WASM module (compass.wasm) computes projected primitives; this file only paints them.

const $ = (id) => document.getElementById(id);
const canvas = $('view');
const ctx = canvas.getContext('2d');

const DEFAULT_VIEW = { yaw: 0.6, pitch: -0.4, zoom: 1 };

const state = {
  field: [21000, 0, -41000],   // nT, device frame
  range: 100000,               // full scale per axis, nT
  yaw: DEFAULT_VIEW.yaw,
  pitch: DEFAULT_VIEW.pitch,
  zoom: DEFAULT_VIEW.zoom,
  spin: false,
  transparency: 0.8,
  fg: '#e8eef7',
  res: '#ffb020',
  bubble: '#4fb3ff',
  bg: '#0b1220',
  layers: { arrows: true, resultant: true, bubbleWire: true, bubbleFill: true, extents: true, ghost: true, labels: true },
};

// Must match the F_* bits in src/lib.rs
const LAYER_BITS = { arrows: 1, resultant: 2, bubbleWire: 4, bubbleFill: 8, extents: 16, ghost: 32, labels: 64 };
const AXES = ['x', 'y', 'z'];

let wasm = null;
let dirty = true;
let drag = null;

// ---------------------------------------------------------------- helpers

const clamp = (v, lo, hi) => Math.min(hi, Math.max(lo, v));
const fmtNT = (v) => Math.round(v).toLocaleString('en-US');

function hexToRgb(hex) {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function rgba(hex, a) {
  const [r, g, b] = hexToRgb(hex);
  return `rgba(${r},${g},${b},${clamp(a, 0, 1).toFixed(3)})`;
}

function flagsValue() {
  let f = 0;
  for (const [name, bit] of Object.entries(LAYER_BITS)) if (state.layers[name]) f |= bit;
  return f;
}

// ---------------------------------------------------------------- drawing

function strokeColour(style, opacity) {
  if (style === 3) return rgba(state.res, opacity);
  if (style === 5) return rgba(state.bubble, 0.12 + 0.8 * (1 - state.transparency));
  if (style === 6) return rgba(state.fg, 0.55);
  if (style === 7) return rgba(state.fg, 0.25);
  return rgba(state.fg, opacity);
}

function fillColour(style) {
  return style === 3 ? state.res : state.fg;
}

function drawOps(ops, n) {
  const LABELS = [
    `Bx ${fmtNT(state.field[0])}`,
    `By ${fmtNT(state.field[1])}`,
    `Bz ${fmtNT(state.field[2])}`,
    `|B| ${fmtNT(wasm.readout(0))}`,
  ];
  ctx.font = '600 12px system-ui, sans-serif';
  ctx.textBaseline = 'middle';

  for (let i = 0; i < n; i++) {
    const o = i * 9;
    const kind = ops[o];
    const x1 = ops[o + 1], y1 = ops[o + 2];
    const x2 = ops[o + 3], y2 = ops[o + 4];
    const x3 = ops[o + 5], y3 = ops[o + 6];
    const style = ops[o + 7];

    if (kind === 0) {                                  // line
      ctx.setLineDash([]);
      ctx.strokeStyle = style <= 3 ? fillColour(style) : strokeColour(style, 1);
      ctx.lineWidth = style === 3 ? 3 : style <= 2 ? 2.5 : 1;
      ctx.lineCap = 'round';
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();
    } else if (kind === 1) {                           // arrow head
      ctx.fillStyle = fillColour(style);
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.lineTo(x3, y3);
      ctx.closePath();
      ctx.fill();
    } else if (kind === 2) {                           // bubble fill
      const r = x2;
      const alpha = 0.04 + 0.35 * (1 - state.transparency);
      const g = ctx.createRadialGradient(x1 - r * 0.3, y1 - r * 0.3, r * 0.1, x1, y1, r);
      g.addColorStop(0, rgba('#ffffff', alpha * 0.9));
      g.addColorStop(1, rgba(state.bubble, alpha));
      ctx.fillStyle = g;
      ctx.beginPath();
      ctx.arc(x1, y1, r, 0, Math.PI * 2);
      ctx.fill();
    } else if (kind === 3) {                           // dashed line
      ctx.setLineDash([4, 4]);
      ctx.strokeStyle = strokeColour(style, 1);
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();
    } else if (kind === 4) {                           // label
      ctx.setLineDash([]);
      ctx.fillStyle = state.fg;
      ctx.fillText(LABELS[style - 8] ?? '', x1, y1);
    }
  }
  ctx.setLineDash([]);
}

function render() {
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth;
  const h = canvas.clientHeight;
  if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = state.bg;
  ctx.fillRect(0, 0, w, h);

  wasm.set_field(...state.field);
  wasm.set_range(state.range);
  wasm.set_view(state.yaw, state.pitch, state.zoom);
  wasm.set_flags(flagsValue());
  const n = wasm.build(w, h);
  const ops = new Float32Array(wasm.memory.buffer, wasm.ops_ptr(), n * 9);
  drawOps(ops, n);

  updateReadout();
  dirty = false;
}

function updateReadout() {
  const mag = wasm.readout(0);
  $('mag').textContent = `${fmtNT(mag)} nT`;
  $('magUT').textContent = `${(mag / 1000).toFixed(3)} µT`;
  $('elev').textContent = `${wasm.readout(1).toFixed(1)}°`;
  $('azim').textContent = `${wasm.readout(2).toFixed(1)}°`;
  $('warn').style.display = wasm.readout(3) ? 'block' : 'none';
}

// ---------------------------------------------------------------- UI sync

function syncFieldInputs() {
  AXES.forEach((a, i) => {
    $(`s${a}`).min = -state.range;
    $(`s${a}`).max = state.range;
    $(`s${a}`).value = state.field[i];
    $(`n${a}`).value = Math.round(state.field[i]);
  });
}

function syncViewInputs() {
  $('yaw').value = state.yaw;
  $('pitch').value = state.pitch;
  $('zoom').value = state.zoom;
  $('yawVal').textContent = `${(state.yaw * 180 / Math.PI).toFixed(0)}°`;
  $('pitchVal').textContent = `${(state.pitch * 180 / Math.PI).toFixed(0)}°`;
  $('zoomVal').textContent = `${state.zoom.toFixed(2)}×`;
}

function syncAppearanceInputs() {
  $('fg').value = state.fg;
  $('res').value = state.res;
  $('bubble').value = state.bubble;
  $('bg').value = state.bg;
  $('transp').value = Math.round(state.transparency * 100);
  $('transpVal').textContent = `${Math.round(state.transparency * 100)}%`;
  document.querySelectorAll('[data-layer]').forEach((el) => { el.checked = state.layers[el.dataset.layer]; });
  $('range').value = String(state.range);
}

function requestRender() { dirty = true; }

function setField(i, value) {
  state.field[i] = clamp(Number(value) || 0, -state.range, state.range);
  syncFieldInputs();
  requestRender();
}

function setPreset(name) {
  if (name === 'zero') state.field = [0, 0, 0];
  if (name === 'earth') state.field = [21000, 0, -41000];
  if (name === 'diag') state.field = [0.6, -0.3, 0.5].map((v) => v * state.range);
  if (name === 'random') state.field = [0, 1, 2].map(() => Math.round((Math.random() * 2 - 1) * state.range * 0.8));
  syncFieldInputs();
  requestRender();
}

function resetView() {
  Object.assign(state, DEFAULT_VIEW);
  syncViewInputs();
  requestRender();
}

function exportPng() {
  canvas.toBlob((blob) => {
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = 'field-compass.png';
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  }, 'image/png');
}

// ---------------------------------------------------------------- bindings

function bindUI() {
  AXES.forEach((a, i) => {
    $(`s${a}`).addEventListener('input', (e) => setField(i, e.target.value));
    $(`n${a}`).addEventListener('change', (e) => setField(i, e.target.value));
  });

  $('range').addEventListener('change', (e) => {
    state.range = Number(e.target.value);
    state.field = state.field.map((v) => clamp(v, -state.range, state.range));
    syncFieldInputs();
    requestRender();
  });

  $('fg').addEventListener('input', (e) => { state.fg = e.target.value; requestRender(); });
  $('res').addEventListener('input', (e) => { state.res = e.target.value; requestRender(); });
  $('bubble').addEventListener('input', (e) => { state.bubble = e.target.value; requestRender(); });
  $('bg').addEventListener('input', (e) => { state.bg = e.target.value; requestRender(); });
  $('transp').addEventListener('input', (e) => {
    state.transparency = Number(e.target.value) / 100;
    $('transpVal').textContent = `${e.target.value}%`;
    requestRender();
  });

  $('yaw').addEventListener('input', (e) => { state.yaw = Number(e.target.value); syncViewInputs(); requestRender(); });
  $('pitch').addEventListener('input', (e) => { state.pitch = Number(e.target.value); syncViewInputs(); requestRender(); });
  $('zoom').addEventListener('input', (e) => { state.zoom = Number(e.target.value); syncViewInputs(); requestRender(); });

  document.querySelectorAll('[data-layer]').forEach((el) => {
    el.addEventListener('change', () => { state.layers[el.dataset.layer] = el.checked; requestRender(); });
  });

  document.querySelectorAll('[data-preset]').forEach((el) => {
    el.addEventListener('click', () => { setPreset(el.dataset.preset); closeMenus(); });
  });

  document.querySelectorAll('[data-action]').forEach((el) => {
    el.addEventListener('click', () => { runAction(el.dataset.action); closeMenus(); });
  });

  // Menus
  document.querySelectorAll('[data-menu] > button').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const menu = btn.parentElement;
      const wasOpen = menu.classList.contains('open');
      closeMenus();
      if (!wasOpen) menu.classList.add('open');
    });
  });
  document.addEventListener('click', closeMenus);

  // Pointer rotation and wheel zoom
  canvas.addEventListener('pointerdown', (e) => {
    drag = { x: e.clientX, y: e.clientY };
    canvas.setPointerCapture(e.pointerId);
  });
  canvas.addEventListener('pointermove', (e) => {
    if (!drag) return;
    state.yaw += (e.clientX - drag.x) * 0.008;
    state.pitch = clamp(state.pitch + (e.clientY - drag.y) * 0.008, -1.5, 1.5);
    drag = { x: e.clientX, y: e.clientY };
    syncViewInputs();
    requestRender();
  });
  canvas.addEventListener('pointerup', () => { drag = null; });
  canvas.addEventListener('pointercancel', () => { drag = null; });
  canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    state.zoom = clamp(state.zoom * Math.exp(-e.deltaY * 0.001), 0.2, 4);
    syncViewInputs();
    requestRender();
  }, { passive: false });

  new ResizeObserver(requestRender).observe(canvas);
}

function runAction(action) {
  switch (action) {
    case 'export': exportPng(); break;
    case 'reset-view': resetView(); break;
    case 'spin': state.spin = !state.spin; break;
    case 'toggle-panel': $('panel').classList.toggle('hidden'); requestRender(); break;
    case 'about':
      alert('3-Axis Field Compass\n\nGeometry: WebAssembly (Rust), compiled to wasm32.\nRendering: Canvas 2D.\nUnits: nanotesla (nT).');
      break;
  }
}

function closeMenus() {
  document.querySelectorAll('[data-menu].open').forEach((m) => m.classList.remove('open'));
}

// ---------------------------------------------------------------- loop

function frame() {
  if (state.spin) {
    state.yaw += 0.01;
    syncViewInputs();
    dirty = true;
  }
  if (dirty) render();
  requestAnimationFrame(frame);
}

async function init() {
  try {
    const response = await fetch('compass.wasm');
    if (!response.ok) throw new Error(`compass.wasm: HTTP ${response.status}`);
    const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), {});
    wasm = instance.exports;
  } catch (err) {
    const box = $('error');
    box.style.display = 'flex';
    box.textContent = `Could not load the WASM module (${err.message}). Serve this folder over HTTP, e.g. python3 -m http.server, and open it from there.`;
    return;
  }
  bindUI();
  syncFieldInputs();
  syncViewInputs();
  syncAppearanceInputs();
  requestAnimationFrame(frame);
}

init();
