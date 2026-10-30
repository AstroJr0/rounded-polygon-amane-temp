## Usage

Every example below is a function that returns a widget. `ShapeCanvas::build()` gives you a normal amane `Rectangle`, so you can add `.on_click(...)`, `.opacity(...)` and the rest to it.

```rust
use amane::*;
use crate::rounded_polygon_amane::presets::material_shapes;
use crate::rounded_polygon_amane::{
    custom_polygon, CornerRounding, Offset, PointNRound, RoundedPolygon, ShapeCanvas,
};
```

A morph happens by itself whenever the polygon you hand to `rounded_polygon` changes between redraws. To see one, you need some state to change. The examples use a small [service](https://mystiafin.github.io/amane/services.html) with one switch:

```rust
struct Toggle {
    on: bool,
}

impl Service for Toggle {
    fn new() -> Self {
        Self { on: false }
    }
}
```
