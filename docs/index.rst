3-Axis Field Compass
====================

An interactive visualisation of a three-axis magnetic field measurement in
nanotesla (nT). Three orthogonal arrows meet at a central point; their lengths
and signs are the measured Bx, By and Bz components. A resultant arrow shows the
total vector, and a transparent bubble marks the full-scale range.

The geometry is computed by a Rust module compiled to WebAssembly. A small
browser harness supplies the menu, toolbar and controls, and draws the result
with Canvas 2D.

.. toctree::
   :maxdepth: 2
   :caption: Contents

   overview
   running
   harness
   wasm-api
   building
   contributing


Indices
-------

* :ref:`genindex`
* :ref:`search`
