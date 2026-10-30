# Morphing rounded polygons in Rust

A shape library made *specifically* for [amane](https://github.com/MystiaFin/amane). Draw rounded polygons, and watch them morph into each other.

It includes the 35 [Material Design shapes](https://m3.material.io/styles/shape/overview-principles#579dd4ba-39f3-4e60-bd9b-1d97ed6ef1bf). You can also make your own custom polygons!

*pstt.. heres the [cheatsheet](./helps/CHEATSHEET.md), and how to use [SVG paths](./helps/using_svgs.md)*

## Credits

This is an amane port of:

- [Androidx's shape library](https://developer.android.com/jetpack/androidx/releases/graphics), the original.
- [Knugel's TypeScript port](https://github.com/Knugel/rounded-polygon-ts) of it. also thanks to [end-4's QML port](https://github.com/end-4/rounded-polygon-qmljs).
- and [amane](https://github.com/Mystiafin/amane), which is an awesome desktop UI framework.. Thank u, MystiaFin.

## Documentation 
Documentation is available [here](https://astrojr0.github.io/rounded_polygon_amane)




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

## License

[Apache-2.0](./LICENSE), like the projects above.
