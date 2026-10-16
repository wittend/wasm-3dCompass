WebAssembly API
===============

The module ``compass.wasm`` is built from ``src/lib.rs``. It exports plain C-ABI
functions and has no imports, so it can be instantiated with an empty import
object. All angles are in radians and all field values in nanotesla.

Exported functions
------------------

``set_field(x: f32, y: f32, z: f32)``
   Set the measured field components Bx, By, Bz (nT). Also updates the readout.

``set_range(range_nt: f32)``
   Set the full-scale range (nT) for each component. This is also the bubble
   radius. Values below 1 are clamped to 1. Also updates the readout.

``set_declination(deg: f32)``
   Set the magnetic declination in degrees, positive east, clamped to ±180.
   It turns the compass rose so its N mark points to true north. Also updates
   the readout.

``set_view(yaw: f32, pitch: f32, zoom: f32)``
   Set the point of view. Pitch is clamped to ±1.5 rad and zoom to 0.2–4.

``set_flags(flags: u32)``
   Set which layers are generated. Bits are listed below.

``build(w: f32, h: f32) -> u32``
   Generate the drawing list for a canvas of ``w`` × ``h`` CSS pixels. Returns
   the number of operations. Call this once per frame after setting the state.

``ops_ptr() -> *const f32``
   Pointer into linear memory to the drawing list written by ``build``.

``readout(i: u32) -> f32``
   Read a derived value (see below).

Layer flags
-----------

=============  =====  ===============================
Name           Bit    Layer
=============  =====  ===============================
F_ARROWS       1      Component arrows
F_RESULTANT    2      Resultant arrow
F_BUBBLE_WIRE  4      Bubble great-circle outline
F_BUBBLE_FILL  8      Bubble body
F_EXTENTS      16     Extent lines
F_GHOST        32     Full-scale axes
F_LABELS       64     Value labels
F_ROSE         128    Compass rose (XY plane)
F_NORTH_ARROW  256    Red arrow to true north
=============  =====  ===============================

The default is ``0x1ff`` (all layers on).

Readout indices
---------------

=====  ==============================================================
Index  Value
=====  ==============================================================
0      \|B\| in nT
1      Elevation in degrees, ``asin(Bz / |B|)``
2      Azimuth in degrees, ``atan2(By, Bx)``
3      1.0 if any component exceeds the full-scale range, else 0.0
4      Direction of true north from +X towards +Y, in degrees, in (-180, 180]
=====  ==============================================================

Index 4 is ``atan2(By, Bx) - declination``, using the horizontal field as
magnetic north. With no horizontal field (below 1 nT) it uses +X as the
magnetic reference.

Drawing list
------------

``build`` writes ``n`` records of nine ``f32`` values each, sorted from back to
front. Read them with ``new Float32Array(memory.buffer, ops_ptr(), n * 9)``.

Each record is ``[kind, x1, y1, x2, y2, x3, y3, style, depth]``.

Kinds:

=  ================  ==================================================
0  line              (x1, y1) to (x2, y2)
1  filled triangle   (x1, y1), (x2, y2), (x3, y3); arrow head
2  filled circle     centre (x1, y1), radius x2; bubble body
3  dashed line       (x1, y1) to (x2, y2)
4  label             anchor (x1, y1); text chosen by style
=  ================  ==================================================

Styles:

=====  ====================================================
Style  Meaning
=====  ====================================================
0      X component arrow (foreground)
1      Y component arrow (foreground)
2      Z component arrow (foreground)
3      Resultant arrow
4      Bubble body
5      Bubble wire
6      Extent line
7      Full-scale axis
8      Label: Bx
9      Label: By
10     Label: Bz
11     Label: \|B\|
12     Label: N (compass rose)
13     Label: E (compass rose)
14     Label: S (compass rose)
15     Label: W (compass rose)
16     Compass rose face (translucent triangle)
17     Compass rose rim and ticks
18     Red arrow to true north (fixed red, drawn as a line and a head)
=====  ====================================================

``depth`` is the view-space z of the element. Larger values are nearer the
viewer. The harness does not need to sort; the list is already ordered.

Example
-------

.. code-block:: javascript

   const { instance } = await WebAssembly.instantiate(
     await (await fetch("compass.wasm")).arrayBuffer(), {});
   const wasm = instance.exports;

   wasm.set_field(21000, 0, -41000);   // nT
   wasm.set_range(100000);
   wasm.set_view(0.6, -0.4, 1.0);
   wasm.set_declination(12);           // true north 12° east of magnetic north
   wasm.set_flags(0x1ff);

   const n = wasm.build(800, 600);
   const ops = new Float32Array(wasm.memory.buffer, wasm.ops_ptr(), n * 9);
   // ops[i * 9 + 0] is the kind of record i, and so on.
