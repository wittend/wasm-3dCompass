//! Geometry engine for the 3-axis magnetic vector view.
//!
//! The field (in nanotesla) is normalised by a full-scale range and mapped to
//! three orthogonal arrows meeting at the origin, a resultant arrow, and a
//! transparent bubble whose radius is the full-scale extent. This module only
//! computes projected drawing primitives; the JavaScript harness rasterises
//! them with Canvas 2D.
//!
//! Each drawing op is 9 f32s: [kind, x1, y1, x2, y2, x3, y3, style, depth].
//! Ops are returned sorted back-to-front so the harness can paint in order.

use core::ptr::addr_of_mut;

const MAX_OPS: usize = 1024;
const FLOATS: usize = 9;
const CAM: f32 = 4.0; // camera distance, in bubble radii
const R: f32 = 1.0; // bubble radius in world units (= full-scale range)
const ARC_SEGS: usize = 64;

// Op kinds
const K_LINE: f32 = 0.0;
const K_TRI: f32 = 1.0;
const K_CIRCLE: f32 = 2.0; // centre in (x1,y1), radius in x2
const K_DASH: f32 = 3.0;
const K_LABEL: f32 = 4.0; // anchor in (x1,y1); style = S_LABEL_BASE + id

// Style ids (colours and widths are chosen by the harness)
const S_X: f32 = 0.0;
const S_Y: f32 = 1.0;
const S_Z: f32 = 2.0;
const S_RES: f32 = 3.0;
const S_BUB_FILL: f32 = 4.0;
const S_BUB_WIRE: f32 = 5.0;
const S_EXTENT: f32 = 6.0;
const S_GHOST: f32 = 7.0;
const S_LABEL_BASE: f32 = 8.0; // 8=X, 9=Y, 10=Z, 11=|B|, 12..15 = N, E, S, W
const S_ROSE_FILL: f32 = 16.0;
const S_ROSE_LINE: f32 = 17.0;
const S_NORTH: f32 = 18.0; // red arrow to true north

// Visibility flags
pub const F_ARROWS: u32 = 1 << 0;
pub const F_RESULTANT: u32 = 1 << 1;
pub const F_BUBBLE_WIRE: u32 = 1 << 2;
pub const F_BUBBLE_FILL: u32 = 1 << 3;
pub const F_EXTENTS: u32 = 1 << 4;
pub const F_GHOST: u32 = 1 << 5;
pub const F_LABELS: u32 = 1 << 6;
pub const F_ROSE: u32 = 1 << 7;
pub const F_NORTH_ARROW: u32 = 1 << 8;

struct State {
    b: [f32; 3],
    range: f32,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    flags: u32,
    decl_deg: f32,
    ops: [[f32; FLOATS]; MAX_OPS],
    n: usize,
    readout: [f32; 5],
}

static mut ST: State = State {
    b: [0.0; 3],
    range: 100_000.0,
    yaw: 0.6,
    pitch: -0.4,
    zoom: 1.0,
    flags: 0x1ff,
    decl_deg: 0.0,
    ops: [[0.0; FLOATS]; MAX_OPS],
    n: 0,
    readout: [0.0; 5],
};

fn st() -> &'static mut State {
    // Single-threaded wasm module; the harness never re-enters during a call.
    unsafe { &mut *addr_of_mut!(ST) }
}

/// Orthographic-ish perspective projection: yaw about Y, then pitch about X.
struct Screen {
    cx: f32,
    cy: f32,
    s: f32,
    cyaw: f32,
    syaw: f32,
    cpit: f32,
    spit: f32,
}

impl Screen {
    fn new(w: f32, h: f32, yaw: f32, pitch: f32, zoom: f32) -> Self {
        Screen {
            cx: w * 0.5,
            cy: h * 0.5,
            s: w.min(h) * 0.36 * zoom,
            cyaw: yaw.cos(),
            syaw: yaw.sin(),
            cpit: pitch.cos(),
            spit: pitch.sin(),
        }
    }

    /// Returns (screen x, screen y) and view-space depth (larger = nearer).
    fn project(&self, p: [f32; 3]) -> ([f32; 2], f32) {
        let x = p[0] * self.cyaw + p[2] * self.syaw;
        let z1 = -p[0] * self.syaw + p[2] * self.cyaw;
        let y = p[1] * self.cpit - z1 * self.spit;
        let z = p[1] * self.spit + z1 * self.cpit;
        let k = CAM / (CAM - z);
        ([self.cx + x * k * self.s, self.cy - y * k * self.s], z)
    }
}

fn push(st: &mut State, kind: f32, a: [f32; 2], b: [f32; 2], c: [f32; 2], style: f32, depth: f32) {
    if st.n >= MAX_OPS {
        return;
    }
    st.ops[st.n] = [kind, a[0], a[1], b[0], b[1], c[0], c[1], style, depth];
    st.n += 1;
}

fn segment(st: &mut State, scr: &Screen, p: [f32; 3], q: [f32; 3], kind: f32, style: f32) {
    let (a, za) = scr.project(p);
    let (b, zb) = scr.project(q);
    push(st, kind, a, b, [0.0; 2], style, (za + zb) * 0.5);
}

/// A shaft from `from` to `to` with a triangular head at `to`.
fn arrow(st: &mut State, scr: &Screen, from: [f32; 3], to: [f32; 3], style: f32) {
    let (a, za) = scr.project(from);
    let (b, zb) = scr.project(to);
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let head_len = 0.09 * scr.s;
    let head_w = 0.04 * scr.s;
    let base = [b[0] - ux * head_len, b[1] - uy * head_len];
    let perp = [-uy * head_w, ux * head_w];
    let depth = (za + zb) * 0.5;
    push(st, K_LINE, a, base, [0.0; 2], style, depth);
    push(
        st,
        K_TRI,
        b,
        [base[0] + perp[0], base[1] + perp[1]],
        [base[0] - perp[0], base[1] - perp[1]],
        style,
        depth + 0.001,
    );
}

/// Direction of true north in the sensor's XY plane, as a maths angle (radians)
/// measured from +X towards +Y.
///
/// Magnetic north is taken as the direction of the horizontal field. If the
/// horizontal field is below 1 nT (no usable reference), +X is used instead.
/// True north lies `decl_deg` east of magnetic north; east is clockwise seen
/// from +Z, so true north is the magnetic angle minus the declination.
fn north_angle(b: [f32; 3], decl_deg: f32) -> f32 {
    let bh = (b[0] * b[0] + b[1] * b[1]).sqrt();
    let magnetic = if bh >= 1.0 { b[1].atan2(b[0]) } else { 0.0 };
    magnetic - decl_deg.to_radians()
}

fn readout_from(b: [f32; 3], range: f32, decl_deg: f32) -> [f32; 5] {
    let mag = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    let (elev, azim) = if mag > 0.0 {
        ((b[2] / mag).clamp(-1.0, 1.0).asin().to_degrees(), b[1].atan2(b[0]).to_degrees())
    } else {
        (0.0, 0.0)
    };
    let saturated = if b.iter().any(|c| c.abs() > range) { 1.0 } else { 0.0 };
    let mut north = north_angle(b, decl_deg).to_degrees() % 360.0;
    if north > 180.0 {
        north -= 360.0;
    } else if north < -180.0 {
        north += 360.0;
    }
    [mag, elev, azim, saturated, north]
}

/// Point on the circle of radius `r` in the XY plane at maths angle `t`.
fn rim_point(t: f32, r: f32) -> [f32; 3] {
    [r * t.cos(), r * t.sin(), 0.0]
}

/// Transparent compass rose lying in the XY plane at the bubble's equator.
/// Compass bearings run clockwise from true north, so the maths angle of a
/// bearing `b` is `north - b`.
fn build_rose(st: &mut State, scr: &Screen, north: f32) {
    const SEGS: usize = 64;
    let centre = [0.0f32; 3];
    let (c, zc) = scr.project(centre);

    // Translucent face: a fan of triangles from the centre.
    for k in 0..SEGS {
        let t0 = core::f32::consts::TAU * k as f32 / SEGS as f32;
        let t1 = core::f32::consts::TAU * (k + 1) as f32 / SEGS as f32;
        let (p0, z0) = scr.project(rim_point(t0, R));
        let (p1, z1) = scr.project(rim_point(t1, R));
        push(st, K_TRI, c, p0, p1, S_ROSE_FILL, (zc + z0 + z1) / 3.0);
    }

    // Rim.
    for k in 0..SEGS {
        let t0 = core::f32::consts::TAU * k as f32 / SEGS as f32;
        let t1 = core::f32::consts::TAU * (k + 1) as f32 / SEGS as f32;
        segment(st, scr, rim_point(t0, R), rim_point(t1, R), K_LINE, S_ROSE_LINE);
    }

    // Ticks every 10 degrees: longest at the cardinals, medium every 30.
    for i in 0..36 {
        let bearing = (i as f32) * 10.0_f32.to_radians();
        let t = north - bearing;
        let len = if i % 9 == 0 {
            0.12
        } else if i % 3 == 0 {
            0.07
        } else {
            0.035
        };
        segment(st, scr, rim_point(t, R * (1.0 - len)), rim_point(t, R), K_LINE, S_ROSE_LINE);
    }

    // N, E, S, W just outside the rim.
    for (idx, bearing_deg) in [(4.0_f32, 0.0_f32), (5.0, 90.0), (6.0, 180.0), (7.0, 270.0)] {
        let t = north - bearing_deg.to_radians();
        let (p, z) = scr.project(rim_point(t, R * 1.16));
        push(st, K_LABEL, p, [0.0; 2], [0.0; 2], S_LABEL_BASE + idx, z + 1000.0);
    }
}

fn build_ops(w: f32, h: f32) -> usize {
    let st = st();
    st.n = 0;
    let flags = st.flags;
    let scr = Screen::new(w, h, st.yaw, st.pitch, st.zoom);
    let north = north_angle(st.b, st.decl_deg);

    // Normalised components, clamped to the bubble.
    let mut v = [0.0f32; 3];
    for i in 0..3 {
        v[i] = (st.b[i] / st.range).clamp(-1.0, 1.0);
    }
    let mut d = v; // resultant display vector, kept inside the bubble
    let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if dl > R {
        for c in d.iter_mut() {
            *c *= R / dl;
        }
    }
    let origin = [0.0f32; 3];
    let ex = [v[0], 0.0, 0.0];
    let ey = [0.0, v[1], 0.0];
    let ez = [0.0, 0.0, v[2]];
    let tips = [ex, ey, ez];
    let axis_styles = [S_X, S_Y, S_Z];

    // Bubble fill (drawn first, behind everything).
    if flags & F_BUBBLE_FILL != 0 {
        // Silhouette radius of a sphere of radius R seen from distance CAM.
        let rad = R * CAM / (CAM * CAM - R * R).sqrt() * scr.s;
        push(st, K_CIRCLE, [scr.cx, scr.cy], [rad, 0.0], [0.0; 2], S_BUB_FILL, -1.0e9);
    }

    // Bubble wireframe: three great circles.
    if flags & F_BUBBLE_WIRE != 0 {
        for k in 0..ARC_SEGS {
            let t0 = core::f32::consts::TAU * k as f32 / ARC_SEGS as f32;
            let t1 = core::f32::consts::TAU * (k + 1) as f32 / ARC_SEGS as f32;
            let (c0, s0) = (t0.cos() * R, t0.sin() * R);
            let (c1, s1) = (t1.cos() * R, t1.sin() * R);
            segment(st, &scr, [c0, s0, 0.0], [c1, s1, 0.0], K_LINE, S_BUB_WIRE);
            segment(st, &scr, [c0, 0.0, s0], [c1, 0.0, s1], K_LINE, S_BUB_WIRE);
            segment(st, &scr, [0.0, c0, s0], [0.0, c1, s1], K_LINE, S_BUB_WIRE);
        }
    }

    // Ghost full-scale axes, from -R to +R.
    if flags & F_GHOST != 0 {
        segment(st, &scr, [-R, 0.0, 0.0], [R, 0.0, 0.0], K_DASH, S_GHOST);
        segment(st, &scr, [0.0, -R, 0.0], [0.0, R, 0.0], K_DASH, S_GHOST);
        segment(st, &scr, [0.0, 0.0, -R], [0.0, 0.0, R], K_DASH, S_GHOST);
    }

    // Extent lines: from the resultant tip to each coordinate plane.
    if flags & F_EXTENTS != 0 {
        segment(st, &scr, d, [0.0, d[1], d[2]], K_DASH, S_EXTENT);
        segment(st, &scr, d, [d[0], 0.0, d[2]], K_DASH, S_EXTENT);
        segment(st, &scr, d, [d[0], d[1], 0.0], K_DASH, S_EXTENT);
    }

    // Component arrows.
    if flags & F_ARROWS != 0 {
        for i in 0..3 {
            if tips[i][i].abs() > 1.0e-4 {
                arrow(st, &scr, origin, tips[i], axis_styles[i]);
            }
        }
    }

    // Resultant arrow.
    if flags & F_RESULTANT != 0 && dl > 1.0e-4 {
        arrow(st, &scr, origin, d, S_RES);
    }

    // Compass rose (in the XY plane, under the arrows).
    if flags & F_ROSE != 0 {
        build_rose(st, &scr, north);
    }

    // Red arrow to true north, just inside the rim. Its depth is pushed
    // forward so it is not hidden by the translucent rose face.
    if flags & F_NORTH_ARROW != 0 {
        let before = st.n;
        arrow(st, &scr, origin, rim_point(north, R * 0.92), S_NORTH);
        for i in before..st.n {
            st.ops[i][8] += 500.0;
        }
    }

    // Labels (drawn last, nearest).
    if flags & F_LABELS != 0 {
        for i in 0..3 {
            if tips[i][i].abs() > 1.0e-4 {
                let (p, z) = scr.project(tips[i]);
                push(st, K_LABEL, [p[0] + 10.0, p[1] - 10.0], [0.0; 2], [0.0; 2], S_LABEL_BASE + i as f32, z + 1000.0);
            }
        }
        if dl > 1.0e-4 {
            let (p, z) = scr.project(d);
            push(st, K_LABEL, [p[0] + 10.0, p[1] - 10.0], [0.0; 2], [0.0; 2], S_LABEL_BASE + 3.0, z + 1000.0);
        }
    }

    // Back-to-front (insertion sort by depth; n is small).
    let n = st.n;
    for i in 1..n {
        let cur = st.ops[i];
        let mut j = i;
        while j > 0 && st.ops[j - 1][8] > cur[8] {
            st.ops[j] = st.ops[j - 1];
            j -= 1;
        }
        st.ops[j] = cur;
    }
    n
}

// ---------------------------------------------------------------- exports

#[no_mangle]
pub extern "C" fn set_field(x: f32, y: f32, z: f32) {
    let st = st();
    st.b = [x, y, z];
    st.readout = readout_from(st.b, st.range, st.decl_deg);
}

#[no_mangle]
pub extern "C" fn set_range(range_nt: f32) {
    let st = st();
    st.range = range_nt.max(1.0);
    st.readout = readout_from(st.b, st.range, st.decl_deg);
}

/// Magnetic declination in degrees, positive east. Rotates the compass rose so
/// that it shows true north in the sensor's XY plane.
#[no_mangle]
pub extern "C" fn set_declination(deg: f32) {
    let st = st();
    st.decl_deg = deg.clamp(-180.0, 180.0);
    st.readout = readout_from(st.b, st.range, st.decl_deg);
}

#[no_mangle]
pub extern "C" fn set_view(yaw: f32, pitch: f32, zoom: f32) {
    let st = st();
    st.yaw = yaw;
    st.pitch = pitch.clamp(-1.5, 1.5);
    st.zoom = zoom.clamp(0.2, 4.0);
}

#[no_mangle]
pub extern "C" fn set_flags(flags: u32) {
    st().flags = flags;
}

/// Rebuilds the display list for a canvas of w x h CSS pixels; returns op count.
#[no_mangle]
pub extern "C" fn build(w: f32, h: f32) -> u32 {
    build_ops(w, h) as u32
}

/// Pointer to `build` output: op count * 9 f32s, back-to-front.
#[no_mangle]
pub extern "C" fn ops_ptr() -> *const f32 {
    unsafe { (*addr_of_mut!(ST)).ops.as_ptr() as *const f32 }
}

/// 0 = |B| nT, 1 = elevation deg, 2 = azimuth deg, 3 = 1 if any component exceeds range,
/// 4 = true-north angle in the XY plane, deg from +X towards +Y (-180..180].
#[no_mangle]
pub extern "C" fn readout(i: u32) -> f32 {
    st().readout[(i as usize).min(4)]
}
