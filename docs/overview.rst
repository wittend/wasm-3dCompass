Overview
========

What is shown
-------------

The display represents one three-axis field measurement.

Component arrows
   Three arrows lie along the X, Y and Z axes and meet at the origin. Each
   arrow's length is the measured component, so a negative value points
   the other way along its axis.

Resultant arrow
   An arrow from the origin to (Bx, By, Bz). Its length is the magnitude of
   the field vector.

Bubble
   A transparent sphere whose radius is the *full-scale range* (default
   ±100,000 nT). It shows the extent over which the components can be read.
   Components beyond full scale are clamped to the bubble wall and a warning
   is shown in the readout.

Extent lines
   Dashed lines from the resultant tip to each coordinate plane. They make the
   component values visible in three dimensions.

Full-scale axes
   Faint dashed lines from -R to +R along each axis, marking the range.

Readout
   \|B\| in nT and µT, elevation (angle above the XY plane) and azimuth
   (angle in the XY plane, from +X towards +Y), in degrees.

Units and conventions
---------------------

* All field values are in nanotesla (nT). 1 µT = 1000 nT.
* The device frame is right-handed: X, Y and Z form the axes as drawn.
* Elevation is ``asin(Bz / |B|)``. Azimuth is ``atan2(By, Bx)``.
* The bubble radius is the full-scale range, not the field magnitude.

Architecture
------------

::

   web/index.html, web/app.js     Harness: UI state, input, Canvas 2D drawing
            |  set_field / set_view / set_flags / build
            v
   web/compass.wasm               Geometry engine (from src/lib.rs)
            |  flat array of drawing primitives, sorted back-to-front
            v
   Canvas 2D                      Painted by the harness

The WebAssembly module holds no drawing state of its own beyond the last
inputs. It projects the geometry and writes drawing primitives into linear
memory. The harness reads them and paints them, so the rendering can be changed
without touching the maths.
