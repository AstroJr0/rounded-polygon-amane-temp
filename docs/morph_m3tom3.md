# Material Shape to Material Shape

This page shows a morph between two ready-made shapes. A click switches the polygon, and the canvas animates the change.

## Example

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

`Toggle` is defined in [Usage](./usage.md).

## Explanation

1. `Toggle::read().on` reads the current state: `false` at first.
2. The closure given to `.rounded_polygon(...)` returns `cookie_9_sided` while `on` is `false`, and `flower` while it is `true`. The keyword `move` gives the closure its own copy of `on`.
3. `.morph_duration(400.0)` lengthens the morph from the default 300 milliseconds to 400.
4. `.on_click(...)` is called on the finished `Rectangle`. Each click inverts the state. amane then redraws the view, and the canvas receives the other polygon.

Because the new polygon differs from the previous one, the canvas starts a morph. Each feature of the cookie is paired with a feature of the flower and moved to its new position. The next click reverses the transition.

The click handler is not part of the library; it only supplies a change of state, so that the morph can be observed.
