### A custom polygon morphing into a custom polygon

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

> [!TIP]
> Polygons with a different number of corners morph fine. The corners are matched up for you.
> This one also turns half a turn on every morph.

First we define a custom polygon, tutorial [here](./docs/def_custom_polygon.md)
Event listener is not needed, it's just to show how it morphs
When the [`on`](./docs/usage.md) is true, the custom polygons `blob(6, 0.3)` is ran. 
Else, `blob(3, 0.2)` takes it's place.
You can add a bezier by adding `.morph_easing()` with an enumeration `easing::TYPE()`
