# 3-Axis Field Compass

An interactive visualisation of a three-axis magnetic field measurement, in nanotesla (nT).

Three orthogonal arrows meet at a central point. Their lengths and signs are the
measured Bx, By and Bz components. A resultant arrow shows the total vector, and a
transparent bubble marks the full-scale range of each component.

- **Geometry:** Rust, compiled to WebAssembly (`wasm32-unknown-unknown`), no dependencies.
- **Harness:** plain HTML, CSS and JavaScript, with a menu bar, toolbar and control panel.
  Rendering uses Canvas 2D.

## Features

- Live update from Bx, By and Bz (sliders or numeric entry).
- Selectable full-scale range (±50,000 to ±500,000 nT).
- Foreground, resultant, bubble and background colours.
- Bubble transparency control.
- Point of view: drag to rotate, wheel to zoom, yaw/pitch/zoom sliders, optional spin.
- Layer toggles for arrows, resultant, bubble wire and fill, extent lines, full-scale axes and labels.
- Transparent compass rose in the XY plane, with a declination input that turns it to true north.
- Readout of |B| in nT and µT, elevation and azimuth.
- PNG export.

## Quick start

The harness loads `compass.wasm` with `fetch`, so it must be served over HTTP.
Opening `index.html` directly from disk will not work.

```sh
git clone https://github.com/wittend/wasm-3dCompass.git
cd wasm-3dCompass/web
python3 -m http.server 8000
```

Then open <http://127.0.0.1:8000/index.html>.

The compiled module is committed at `web/compass.wasm`, so no Rust toolchain is needed to run it.

## Building the WebAssembly module

Requires a Rust toolchain with the `wasm32-unknown-unknown` target:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/wasm_3dcompass.wasm web/compass.wasm
```

## Documentation

Full documentation (Sphinx, Furo theme) is in [`docs/`](docs/) and is published on
Read the Docs. To build it locally:

```sh
python3 -m venv .venv
. .venv/bin/activate
pip install -r docs/requirements.txt
sphinx-build -b html docs docs/_build/html
```

Then open `docs/_build/html/index.html`.

## Repository layout

| Path | Contents |
| --- | --- |
| `src/lib.rs` | WebAssembly geometry engine and exported API |
| `web/index.html`, `web/app.js` | Browser harness (menu, toolbar, controls, drawing) |
| `web/compass.wasm` | Built module used by the harness |
| `docs/` | Sphinx sources for Read the Docs |
| `.readthedocs.yaml` | Read the Docs build configuration |
| `.github/workflows/ci.yml` | CI: builds the WASM module and the docs |

## Licence

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
