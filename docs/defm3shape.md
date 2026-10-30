### Defining a Material shape
> [!NOTE]
> Material Shapes are available directly inside the presets
> which you can get through `material_shape`
Example:

```rust
fn heart() -> Rectangle {
    ShapeCanvas::new()
        .rounded_polygon(material_shapes::heart)
        .color("#f38ba8")
        .size(96.0, 96.0)
        .build()
}
```

Each one is a function in `material_shapes`, so `material_shapes::heart` can be passed straight to `.rounded_polygon(...)`.

All list of shapes are available [here](./helps/shapes.md)
