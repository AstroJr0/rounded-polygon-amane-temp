# Defining a Material Shape

The library contains thirty-five ready-made shapes taken from the [Material Design shape set](https://m3.material.io/styles/shape/overview-principles#579dd4ba-39f3-4e60-bd9b-1d97ed6ef1bf). They are located in the module `presets::material_shapes`.

## Example

```rust
fn heart() -> Rectangle {
    ShapeCanvas::new()
        .rounded_polygon(material_shapes::heart)
        .color("#f38ba8")
        .size(96.0, 96.0)
        .build()
}
```

## Explanation

- `ShapeCanvas::new()` creates a canvas with its default settings.
- `.rounded_polygon(material_shapes::heart)` selects the polygon to draw. The method accepts any function that returns a `RoundedPolygon`. Every shape in `material_shapes` is such a function, so its name can be passed without calling it.
- `.color("#f38ba8")` sets the fill colour, given as a hexadecimal string.
- `.size(96.0, 96.0)` sets the canvas to 96 by 96 pixels. The shape is already normalised to the unit square, and is therefore scaled to fill this size.
- `.build()` converts the canvas into an amane `Rectangle`.

Each shape is built once and cloned afterwards. Calling the same shape repeatedly during redraws is therefore inexpensive.

The complete list of shapes is given in [Available Shapes](./shapes.md).
