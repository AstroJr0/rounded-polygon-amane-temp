# Morphing rounded polygons in Rust

A shape library made *specifically* for [amane](https://github.com/MystiaFin/amane). Draw rounded polygons, and watch them morph into each other.

It includes the 35 [Material Design shapes](https://m3.material.io/styles/shape/overview-principles#579dd4ba-39f3-4e60-bd9b-1d97ed6ef1bf). You can also make your own custom polygons!

*pstt.. heres the [cheatsheet](./helps/CHEATSHEET.md)*

## Credits

This is an amane port of:

- [Androidx's shape library](https://developer.android.com/jetpack/androidx/releases/graphics), the original.
- and [amane](https://github.com/Mystiafin/amane), which is an awesome desktop UI framework.. Thank u, MystiaFin.
- not to mention, This project is heavily inspired by *(though i yoinked some code)* [Knugel's TypeScript port](https://github.com/Knugel/rounded-polygon-ts) of it. also thanks to [end-4's QML port](https://github.com/end-4/rounded-polygon-qmljs).

## Installation

Actually, no installation required! Just clone this repo inside your amane dots and you're good to go:

```sh
git clone https://github.com/AstroJr0/rounded_polygon_amane
```

Then declare it in your `main.rs`:

```rust
mod rounded_polygon_amane;
```

This is where it goes:

```
my-shell/
├── Cargo.toml
└── src/
    ├── main.rs                  ← mod rounded_polygon_amane;
    └── rounded_polygon_amane/   ← this repo
        ├── mod.rs
        └── ...
```

No `Cargo.toml` changes. It only uses `amane`, which your shell *supposed to have*.

## Usage

Every example below is a function that returns a widget. `ShapeCanvas::build()` gives you a normal amane `Rectangle`, so you can add `.on_click(...)`, `.opacity(...)` and the rest to it.

```rust
use amane::*;
use crate::rounded_polygon_amane::presets::material_shapes;
use crate::rounded_polygon_amane::{
    custom_polygon, CornerRounding, Offset, PointNRound, RoundedPolygon, ShapeCanvas,
};
```

A morph happens by itself whenever the polygon you hand to `rounded_polygon` changes between redraws. To see one, you need some state to change. The examples use a small [service](https://mystiafin.github.io/amane/services.html) with one switch:

```rust
struct Toggle {
    on: bool,
}

impl Service for Toggle {
    fn new() -> Self {
        Self { on: false }
    }
}
```

### A material shape

<details>
<summary>Show code</summary>

```rust
fn heart() -> Rectangle {
    ShapeCanvas::new()
        .rounded_polygon(material_shapes::heart)
        .color("#f38ba8")
        .size(96.0, 96.0)
        .build()
}
```

</details>

Each one is a function in `material_shapes`, so `material_shapes::heart` can be passed straight to `.rounded_polygon(...)`.

All list of shapes are available [here](./helps/shapes.md)

### A material shape morphing into another material shape

Click it, and the cookie morphs into a flower and back:

<details>
<summary>Show code</summary>

```rust
fn cookie_to_flower() -> Rectangle {
    let on = Toggle::read().on;

    ShapeCanvas::new()
        .rounded_polygon(move || {
            if on {
                material_shapes::flower()
            } else {
                material_shapes::cookie_9_sided()
            }
        })
        .color("#cba6f7")
        .size(96.0, 96.0)
        .morph_duration(400.0)
        .build()
        .on_click(|_| Toggle::write().on = !on)
}
```

</details>

### A custom polygon

A `PointNRound` is a corner: where it is, and how round it is. Give `custom_polygon` the corners of one slice, and how many times to repeat it around the center:

<details>
<summary>Show code</summary>

```rust
fn star() -> Rectangle {
    // One slice: a sharp tip, then a rounded dip. Repeated 5 times.
    let slice = [
        PointNRound::unrounded(Offset::new(0.5, 0.0)),
        PointNRound::new(Offset::new(0.62, 0.38), CornerRounding::from_radius(0.1)),
    ];

    ShapeCanvas::new()
        .rounded_polygon(|| custom_polygon(&slice, 5, Offset::new(0.5, 0.5), false).normalized())
        .color("#89b4fa")
        .size(96.0, 96.0)
        .build()
}
```

</details>

The last argument mirrors every other slice. Corners take a radius, plus an optional smoothing: `CornerRounding::new(radius, smoothing)`.

### A custom polygon morphing into a custom polygon

<details>
<summary>Show code</summary>

```rust
fn blob(corners: usize, radius: f32) -> impl FnOnce() -> RoundedPolygon {
    move || {
        let slice = [PointNRound::new(
            Offset::new(0.5, 0.0),
            CornerRounding::from_radius(radius),
        )];
        custom_polygon(&slice, corners, Offset::new(0.5, 0.5), false).normalized()
    }
}

fn triangle_to_hexagon() -> Rectangle {
    let on = Toggle::read().on;

    ShapeCanvas::new()
        .rounded_polygon(if on { blob(6, 0.3) } else { blob(3, 0.2) })
        .color("#a6e3a1")
        .size(96.0, 96.0)
        .rotate_on_morph(true)
        .rotation_on_morph(180.0)
        .build()
        .on_click(|_| Toggle::write().on = !on)
}
```

</details>

> [!TIP]
> Polygons with a different number of corners morph fine. The corners are matched up for you.
> This one also turns half a turn on every morph.

### A custom polygon morphing into a material shape

Custom and material polygons are the same type, so they mix freely:

<details>
<summary>Show code</summary>

```rust
fn custom_to_material() -> Rectangle {
    let on = Toggle::read().on;

    ShapeCanvas::new()
        .rounded_polygon(move || {
            if on {
                material_shapes::soft_burst()
            } else {
                custom_polygon(
                    &[PointNRound::new(Offset::new(1.0, 0.5), CornerRounding::from_radius(0.15))],
                    4,
                    Offset::new(0.5, 0.5),
                    false,
                )
                .normalized()
            }
        })
        .color("#fab387")
        .size(96.0, 96.0)
        .build()
        .on_click(|_| Toggle::write().on = !on)
}
```

</details>

Put any of these in a window like any other widget: `LayerWindow::new().width(96.0).height(96.0).child(heart())`.

Want something on top of a shape? Put both in a `Stack`: the shape first, then a `Rectangle` or *something else* holding the `something` i.e. `Text`.
