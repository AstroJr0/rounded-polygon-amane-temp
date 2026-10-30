# Usage

Every example in this documentation is a function that returns a widget. `ShapeCanvas::build()` returns an ordinary amane `Rectangle`; consequently, any method available on a `Rectangle`, such as `.on_click(...)` or `.opacity(...)`, may be called on the result.

## Imports

The examples assume the following imports:

```rust
use amane::*;
use crate::rounded_polygon_amane::presets::material_shapes;
use crate::rounded_polygon_amane::{
    custom_polygon, CornerRounding, Offset, PointNRound, RoundedPolygon, ShapeCanvas,
};
```

The first line brings in amane. The second brings in the module `material_shapes`, which holds the ready-made shapes. The third brings in the items needed to build custom polygons and the canvas itself.

## The Toggle Service

A morph occurs whenever the polygon given to `rounded_polygon` differs from the one given on the previous redraw. To observe a morph, a value that changes is therefore required. The morphing examples use a small [service](https://mystiafin.github.io/amane/services.html) that stores a single boolean:

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

`Toggle::read().on` yields the current value, and `Toggle::write().on = !on` inverts it. A click handler writes the new value; amane then redraws the view, the canvas receives a different polygon, and the morph begins.

The service is only a device for the examples. Any other source of changing state, such as a timer or a system event, serves equally well.
