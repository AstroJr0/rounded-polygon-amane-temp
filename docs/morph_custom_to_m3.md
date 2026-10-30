<h1 align="center">Custom Polygon to Material Shape</h1>

A `custom_polygon` and a `material_shape` are both of type `RoundedPolygon`. They may therefore be used together freely, including as the two ends of a morph.

## Example

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

`Toggle` is defined in [Usage](./usage.md).

## Explanation

While `on` is `false`, the canvas shows a custom polygon: a single rounded corner on the right-hand side of the unit square, repeated four times around the centre. The four copies are 90° apart, which gives a rounded square.

While `on` is `true`, the canvas shows the Material shape `soft_burst`.

Both branches of the `if` produce a `RoundedPolygon`, and both are normalised to the unit square (the Material shape already is). The canvas does not distinguish between their origins; it only compares the polygons and, when they differ, morphs between them.
