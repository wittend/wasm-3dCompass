Using the harness
==================

The page has three parts: a menu bar, a toolbar, and a control panel. The
canvas fills the remaining space. On narrow screens the panel moves below the
canvas.

Menu bar
--------

File
   *Export PNG…* saves the current view.

View
   *Reset point of view* returns yaw, pitch and zoom to their defaults.
   *Toggle spin* rotates the view slowly about the vertical axis.
   *Show / hide controls* collapses the side panel.

Field
   Presets for the measured vector: *Zero field*, *Earth-like* (21,000,
   0, −41,000 nT, \|B\| ≈ 46,065 nT), *Diagonal test* and *Random*.

Help
   *About* shows a short description of the project, the implementation
   (WebAssembly and Canvas 2D) and the units used.

Toolbar
-------

The toolbar repeats the most-used actions and adds one toggle per layer:

* **Arrows**: the three component arrows.
* **Resultant**: the total-field arrow.
* **Bubble wire**: the great-circle outline of the full-scale bubble.
* **Bubble fill**: the translucent bubble body.
* **Extents**: dashed lines from the resultant tip to the coordinate planes.
* **Full-scale axes**: the faint ±R axis lines.
* **Labels**: Bx, By, Bz and \|B\| values at the arrow tips.

Control panel
-------------

Measured field (nT)
   Bx, By and Bz, each as a slider and a number box. The sliders run from
   −range to +range. *Full scale* sets the range and the bubble radius.

Appearance
   * **Foreground**: the component arrows and labels.
   * **Resultant arrow**: the total-field arrow.
   * **Bubble**: the bubble wire and fill.
   * **Background**: the canvas colour.
   * **Bubble transparency**: 0% is opaque, 100% is invisible. It applies to
     the bubble fill and wire only.

Point of view
   * **Yaw** rotates about the vertical axis; **Pitch** tilts the view.
   * **Zoom** scales the drawing (0.2× to 4×).
   * Dragging the canvas rotates the view; the mouse wheel zooms.

Readout
   \|B\| in nT and µT, elevation and azimuth. A warning appears when any
   component exceeds the full-scale range.
