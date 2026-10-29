# Cheat Sheet 🥋

Every function you'll use, on one page. Material shape names are in the [README](../README.md).

All names come from `rounded_polygon_amane`. To bring in the common ones:

```rust
use crate::rounded_polygon_amane::presets::material_shapes;
use crate::rounded_polygon_amane::*;
```

## ShapeCanvas

Draws a polygon and morphs it when the polygon changes. Ends with `.build()`, which gives an amane `Rectangle`.

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `ShapeCanvas::new()` | none | starts a canvas: 100×100, `#685496`, centered, 300 ms morph | `ShapeCanvas::new()` |
| `.rounded_polygon(f)` | `FnOnce() -> RoundedPolygon` | the polygon to draw. A different one than last redraw morphs. | `.rounded_polygon(material_shapes::heart)` |
| `.color(c)` | hex string or `Color` | fill color | `.color("#89b4fa")` or `.color(Color::BLUE)` |
| `.border(w, c)` | `f32`, hex string or `Color` | outline of `w` pixels | `.border(2.0, "#cdd6f4")` |
| `.size(w, h)` | `f32`, `f32` | canvas size in pixels | `.size(96.0, 96.0)` |
| `.anchor(h, v)` | `Anchor`, `Anchor` | where the polygon sits in a bigger canvas: `Start`, `Center` (default), `End` | `.anchor(Anchor::Start, Anchor::End)` |
| `.offset(x, y)` | `f32`, `f32` | moves the polygon, in pixels | `.offset(4.0, 0.0)` |
| `.normalized(b)` | `bool` | `true` (default): polygon spans `0..1` and is scaled up. `false`: it is already in pixels. | `.normalized(false)` |
| `.morph_duration(ms)` | `f32` | how long a morph takes. Default `300.0`. | `.morph_duration(500.0)` |
| `.morph_easing(e)` | `[f32; 4]` or an [easing](#easing) | cubic Bézier `[x1, y1, x2, y2]`. Default is "out back". | `.morph_easing(easing::out_back().overshoot(1.7))` |
| `.rotate_on_morph(b)` | `bool` | turn while morphing. Default `false`. | `.rotate_on_morph(true)` |
| `.rotation_on_morph(deg)` | `f32` | degrees added per morph, clockwise, adding up. Default `360.0`. | `.rotation_on_morph(90.0)` |
| `.easing_on_rotation(e)` | `[f32; 4]` or an [easing](#easing) | cubic Bézier for the turn. Default is "out cubic". | `.easing_on_rotation(easing::out_cubic())` |
| `.view(b)` | `bool` | `false` hides the polygon but keeps the size | `.view(is_open)` |
| `.debug(b)` | `bool` | red dots on every anchor point | `.debug(true)` |
| `.id(name)` | `&str` or `String` | names the canvas. Needed for canvases made in a loop, or shared by several windows. | `.id(format!("shape-{i}"))` |
| `.build()` | none | makes the widget. It's a `Rectangle`, so `.on_click(...)`, `.opacity(...)` and so on work on it. | `.build().on_click(\|_\| ...)` |

## Material shapes

All 35 live in `material_shapes`

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `material_shapes::name()` | none | a ready-made, normalized `RoundedPolygon`. Built once, then cloned. | `material_shapes::cookie_9_sided()` |

## Custom polygons

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `custom_polygon(points, reps, center, mirroring)` | `&[PointNRound]`, `usize`, `Offset`, `bool` | a polygon from corners, repeated around `center` | `custom_polygon(&[corner], 4, Offset::new(0.5, 0.5), false)` |
| `PointNRound::new(at, rounding)` | `Offset`, `CornerRounding` | a rounded corner | `PointNRound::new(Offset::new(1.0, 0.5), CornerRounding::from_radius(0.2))` |
| `PointNRound::unrounded(at)` | `Offset` | a sharp corner | `PointNRound::unrounded(Offset::new(0.5, 0.0))` |
| `do_repeat(points, reps, center, mirroring)` | `&[PointNRound]`, `usize`, `Offset`, `bool` | just the repeating. Gives the corner list `custom_polygon` uses. | `do_repeat(&corners, 6, Offset::new(0.5, 0.5), true)` |

`reps` is how many times the slice repeats around the center. `mirroring` flips every other copy.

## Corner rounding

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `CornerRounding::from_radius(r)` | `f32` | a round corner. `r` is relative to the polygon size. | `CornerRounding::from_radius(0.2)` |
| `CornerRounding::new(r, smoothing)` | `f32`, `f32` | the same, plus smoothing (`0.0` to `1.0`) for a softer curve | `CornerRounding::new(0.2, 0.5)` |
| `CornerRounding::UNROUNDED` | none | a sharp corner | `CornerRounding::UNROUNDED` |

## Built-in polygon makers

For when you don't want to list corners yourself.

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `RoundedPolygon::circle(n, radius, center)` | `usize`, `f32`, `Point` | a circle from `n` points. More points, smoother. | `RoundedPolygon::circle(8, 1.0, Point::default())` |
| `RoundedPolygon::from_num_vertices(n, radius, center, rounding, per_corner)` | `usize`, `f32`, `Point`, `CornerRounding`, `Option<&[CornerRounding]>` | a regular polygon: 3 is a triangle, 5 a pentagon | `RoundedPolygon::from_num_vertices(3, 1.0, Point::new(0.5, 0.5), CornerRounding::from_radius(0.2), None)` |
| `RoundedPolygon::rectangle(w, h, rounding, per_corner, center)` | `f32`, `f32`, `CornerRounding`, `Option<&[CornerRounding]>`, `Point` | a rectangle. `per_corner` goes bottom-right, bottom-left, top-left, top-right. | `RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::from_radius(0.3), None, Point::default())` |
| `RoundedPolygon::star(n, radius, inner, rounding, inner_rounding, per_corner, center)` | `usize`, `f32`, `f32`, `CornerRounding`, `Option<CornerRounding>`, `Option<&[CornerRounding]>`, `Point` | a star with `n` tips. `inner` is the dip radius. | `RoundedPolygon::star(5, 1.0, 0.5, CornerRounding::from_radius(0.1), None, None, Point::default())` |
| `RoundedPolygon::from_vertices(xy, rounding, per_corner, center)` | `&[f32]`, `CornerRounding`, `Option<&[CornerRounding]>`, `Option<Point>` | a polygon from a flat `[x0, y0, x1, y1, ...]` list | `RoundedPolygon::from_vertices(&[0.0, 0.0, 1.0, 0.0, 0.5, 1.0], CornerRounding::UNROUNDED, None, None)` |

## Reshaping a polygon

These return a new polygon. The original is untouched.

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `.normalized()` | none | fits it into `0..1`, centered. Call it last, before drawing. | `polygon.normalized()` |
| `.transformed(f)` | `FnMut(f32, f32) -> Point` | moves every point through `f` | `polygon.transformed(\|x, y\| Point::new(x * 2.0, y))` |
| `.cubics()` | none | the curves, as a slice | `polygon.cubics().len()` |
| `.calculate_bounds(approx)` | `bool` | `[min_x, min_y, max_x, max_y]`. `true` is faster, `false` is exact. | `polygon.calculate_bounds(false)` |
| `.center()` | none | the center point | `polygon.center()` |

## Rotating and scaling

Use these inside `.transformed(...)`.

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `Matrix::IDENTITY` | none | a matrix that changes nothing. Start from this. | `let mut m = Matrix::IDENTITY;` |
| `m.rotate_z(deg)` | `f32` | rotates clockwise, in degrees | `m.rotate_z(45.0)` |
| `m.scale(x, y)` | `f32`, `f32` | stretches | `m.scale(1.0, 0.64)` |
| `m.translate(x, y)` | `f32`, `f32` | moves | `m.translate(0.1, 0.0)` |
| `m.map(at)` | `Offset` | the matrix applied to a point | `m.map(Offset::new(x, y))` |
| `Offset::new(x, y)` | `f32`, `f32` | a position | `Offset::new(0.5, 0.5)` |
| `Point::new(x, y)` | `f32`, `f32` | a position, as `transformed` wants it | `Point::new(o.x, o.y)` |

A whole rotation:

```rust
let mut m = Matrix::IDENTITY;
m.rotate_z(45.0);
let turned = polygon.transformed(|x, y| {
    let o = m.map(Offset::new(x, y));
    Point::new(o.x, o.y)
});
```

## Morph

`ShapeCanvas` morphs for you. Use `Morph` only to draw frames yourself.

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `Morph::new(from, to)` | `&RoundedPolygon`, `&RoundedPolygon` | matches the corners of two polygons. `Err` if they can't be matched. | `let morph = Morph::new(&a, &b)?;` |
| `morph.as_cubics(t)` | `f32` | the curves at progress `t`: `0.0` is `from`, `1.0` is `to` | `morph.as_cubics(0.5)` |

## Easing

Named after Qt Quick's `Easing` types. Each gives the `[f32; 4]` that `.morph_easing(...)` and `.easing_on_rotation(...)` take. Needs `use crate::rounded_polygon_amane::presets::easing;`

| Function | Arguments | What it does | Example |
|---|---|---|---|
| `easing::linear()` | none | constant speed | `easing::linear()` |
| `easing::in_quad()` `out_quad()` `in_out_quad()` `out_in_quad()` | none | quadratic. `in` starts slow, `out` starts fast, `in_out` slow at both ends, `out_in` fast at both ends. | `easing::in_out_quad()` |
| `easing::in_cubic()` `out_cubic()` `in_out_cubic()` `out_in_cubic()` | none | cubic, a bit stronger | `easing::out_cubic()` |
| `easing::in_quart()` `out_quart()` `in_out_quart()` `out_in_quart()` | none | quartic, stronger still | `easing::out_quart()` |
| `easing::in_quint()` `out_quint()` `in_out_quint()` `out_in_quint()` | none | quintic, the strongest polynomial | `easing::in_out_quint()` |
| `easing::in_sine()` `out_sine()` `in_out_sine()` `out_in_sine()` | none | a gentle sine wave | `easing::in_out_sine()` |
| `easing::in_expo()` `out_expo()` `in_out_expo()` `out_in_expo()` | none | exponential: very sharp start or end | `easing::out_expo()` |
| `easing::in_circ()` `out_circ()` `in_out_circ()` `out_in_circ()` | none | along a circle's arc | `easing::in_circ()` |
| `easing::in_back()` `out_back()` `in_out_back()` `out_in_back()` | none, then `.overshoot(f32)` | pulls back or overshoots before settling. Default overshoot `1.70158`. `0.0` is a plain cubic. | `easing::out_back().overshoot(2.5)` |

`out_back()` and the other `back` curves hold a value until they are turned into an array. `.morph_easing(...)` does that for you. To get the array yourself, call `.bezier()`.

**Not included:** `elastic` and `bounce` curves wobble more than once, which one cubic Bézier can't do.

Every curve here is a close match to Qt's, not an exact copy. The `out_in_*` ones are the roughest, because Qt builds them from two segments.
