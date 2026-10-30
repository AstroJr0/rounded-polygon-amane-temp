### Defining a custom polygons

A `PointNRound` is a corner: where it is, and how round it is. Give `custom_polygon` the corners of one slice, and how many times to repeat it around the center:

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

The last argument mirrors every other slice. Corners take a radius, plus an optional smoothing: `CornerRounding::new(radius, smoothing)`.
