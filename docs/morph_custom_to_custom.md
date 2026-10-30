# Custom Polygon to Custom Polygon

This page shows a morph between two custom polygons that differ in the number of corners and in their roundness.

## Example

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

`Toggle` is defined in [Usage](./usage.md). The definition of a custom polygon is explained in [Defining a Custom Polygon](./def_custom_polygon.md).

## Explanation

The function `blob` returns a closure that builds a polygon of `corners` identical, rounded corners. Its slice contains a single corner, so repeating it `corners` times yields a regular polygon: three corners give a triangle, and six give a hexagon.

When `on` is `false`, `blob(3, 0.2)` supplies a triangle with a small radius. When `on` is `true`, `blob(6, 0.3)` supplies a hexagon with a larger radius. The `if` expression selects between two values of the same type, so the argument to `.rounded_polygon(...)` is well typed.

> [!TIP]
> Polygons with different numbers of corners can be morphed. The library pairs the features of the two polygons automatically.

Two further settings appear in the example:

- `.rotate_on_morph(true)` makes the shape turn while it morphs.
- `.rotation_on_morph(180.0)` sets the turn to half a revolution per morph, in degrees, clockwise. The default is 360.

The timing of the morph itself may be changed with `.morph_easing(...)`, which accepts a cubic Bézier curve such as one from `presets::easing`. The timing of the turn is changed with `.easing_on_rotation(...)`.
