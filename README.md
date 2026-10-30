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
