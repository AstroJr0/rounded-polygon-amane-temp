### A material shape morphing into another material shape

Click it, and the cookie morphs into a flower and back:

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

Note: `Toggle` is defined [here](./usage.md)
