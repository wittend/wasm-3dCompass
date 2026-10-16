Running the visualisation
=========================

The harness loads ``compass.wasm`` with ``fetch``, so it has to be served
over HTTP. Opening ``index.html`` straight from the file system will fail with a
message in the display area.

Local server
------------

.. code-block:: sh

   git clone https://github.com/wittend/wasm-3dCompass.git
   cd wasm-3dCompass/web
   python3 -m http.server 8000

Then open http://127.0.0.1:8000/index.html.

The compiled module is committed at ``web/compass.wasm``, so you do not need
a Rust toolchain to run it. To rebuild it, see :doc:`building`.

Browser support
---------------

Any current browser with WebAssembly, ``ResizeObserver`` and Canvas 2D support
(Chrome, Edge, Firefox, Safari). Pointer and wheel events are used for
rotation and zoom, so touch and mouse both work.

Export
------

**File > Export PNG…** (or the toolbar button) saves the current canvas as
``field-compass.png`` at the device pixel ratio of the display.
