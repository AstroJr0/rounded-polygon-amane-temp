# Using Vector paths to define a polygon 🖋️

`from_svg` turns a path string into a `RoundedPolygon`, so `ShapeCanvas` can draw it and morph to it like any other shape.

```rust
use crate::rounded_polygon_amane::*;

let triangle = from_svg("M 0 0 L 1 0 L 0.5 1")?;
```

Short version is in the [cheat sheet](./CHEATSHEET.md#svg-paths).

## What is supported

Three instructions, nothing else.

| Instruction | Numbers | Meaning |
|---|---|---|
| `M x y` | 2 | where the path starts |
| `L x y` | 2 | a straight edge to this point |
| `C x1 y1 x2 y2 x y` | 6 | a cubic Bézier to `x y`, bent by the two handles |

Rules:

- **`M` is first, and only once.** One path is one outline.
- **Case doesn't matter.** `M`, `L`, `C` and `m`, `l`, `c` are the same. This is *not* standard SVG, where lowercase means relative. Here everything is absolute.
- **No `Z`.** The polygon closes itself. A path that ends on its starting point is fine too: the repeated corner is dropped.
- **Separators** are spaces or commas: `m0,0 l1,0` works.
- **Repeats** are allowed: `L 1 0 1 1` is `L 1 0 L 1 1`, and `C` takes any number of 6-number groups.
- **Negative numbers need a space before them.** `10 -5`, not `10-5`.
- Any other letter (`H`, `V`, `Q`, `A`, `S`, `Z`...) is an error. Convert those to `L` and `C` first.

## Coordinates

Numbers are used exactly as written. Nothing is scaled, and there is no viewbox. What you write has to match how the canvas reads it:

| You write | Canvas needs |
|---|---|
| numbers in `0..1` | nothing, this is the default. The canvas scales them to `.size(...)`. |
| pixels | `.normalized(false)` |

```rust
// 0..1
ShapeCanvas::new()
    .rounded_polygon(|| from_svg("M 0.5 0 L 1 1 L 0 1").expect("bad svg"))
    .size(96.0, 96.0)
    .build()

// pixels
ShapeCanvas::new()
    .rounded_polygon(|| from_svg("M 0 0 L 500 0 L 250 300").expect("bad svg"))
    .normalized(false)
    .size(500.0, 300.0)
    .build()
```

Y grows downward, like on screen and in SVG.

**A path from a design tool.** Such a path is usually written in the pixels of its own viewbox, for example 500 by 300. Two approaches are available:

1. Use the numbers as they are, give the canvas `.normalized(false)`, and set `.size(...)` to the viewbox size, as in the second example above.
2. Call `.normalized()` on the result. This scales and moves the polygon to fit the unit square, keeping its proportions and centring it along the shorter side. The default canvas then draws it at any size:

```rust
from_svg("M 0 0 L 500 0 L 250 300").expect("bad svg").normalized()
```

## What happens to curves

A polygon is built from corners, so a `C` can't stay a true curve. It is cut into `CURVE_STEPS` (16) straight pieces, and the last one lands exactly on the end point. The start point is not repeated.

16 is smooth at normal sizes. On a very big canvas the edges may show. Change `CURVE_STEPS` in `svg_utils.rs` if you need more or fewer.

## Errors

`from_svg` gives a `Result<RoundedPolygon, SvgError>`.

| Error | Cause | Example |
|---|---|---|
| `MissingMove` | empty path, or it doesn't start with `m` | `"L 1 1"` |
| `RepeatedMove` | a second `m` | `"M 0 0 M 1 1"` |
| `UnknownCommand(c)` | a letter that isn't `m`, `l` or `c` | `"M 0 0 Q 1 1"` |
| `BadArguments(c)` | wrong amount of numbers | `"M 0 0 L 1"` |
| `BadNumber(text)` | not a number | `"M 0 1.2.3"` |

`SvgError` is in `rounded_polygon_amane::handlers::svg_utils`, and prints a plain message. For a path written in your source, `.expect("...")` is enough, since it can't change. For a path from a user or a file, handle the error.

## Building paths from code

It's a string, so `format!` works. Here a 500×300 triangle goes into `0..1`:

```rust
let w = 500.0;
let path = [(0.0, 0.0), (500.0, 0.0), (250.0, 300.0)]
    .iter()
    .enumerate()
    .map(|(i, (x, y))| format!("{} {} {}", if i == 0 { "M" } else { "L" }, x / w, y / w))
    .collect::<Vec<_>>()
    .join(" ");
// "M 0 0 L 1 0 L 0.5 0.6"
```

## Rounding corners

An SVG path has no rounding, so `from_svg` gives sharp corners. To round some, use the pieces `from_svg` is made of. `move_to`, `line_to` and `curve_to` give `PointNRound`s, and you pick the rounding and call `custom_polygon` yourself.

```rust
use crate::rounded_polygon_amane::handlers::svg_utils::{line_to, move_to};
use crate::rounded_polygon_amane::*;

let soft = CornerRounding::from_radius(0.1);

let corners = [move_to(0.2, 0.2), line_to(0.8, 0.2), line_to(0.5, 0.8)]
    .map(|p| PointNRound::new(p.offset, soft));

let shape = custom_polygon(&corners, 1, Offset::new(0.5, 0.5), false);
```

`curve_to` needs the point the curve starts from, since each call stands alone:

```rust
use crate::rounded_polygon_amane::handlers::svg_utils::{curve_to, line_to, move_to};

let mut corners = vec![move_to(0.2, 0.2), line_to(0.8, 0.2)];

corners.extend(curve_to([0.8, 0.2], [1.0, 0.6], [0.6, 0.9], [0.2, 0.8]));

let shape = custom_polygon(&corners, 1, Offset::new(0.5, 0.5), false);
```

Rounding the many corners a curve makes would only dent it, so leave those sharp and round the real corners (`move_to`, `line_to`).

The `center` argument is the middle of the shape. `from_svg` uses the average of the corners, which is a good pick here too.

## Morphing

A `ShapeCanvas` morphs when the polygon it is given changes, so different paths animate into each other:

```rust
let path = if open { OPEN } else { CLOSED };

ShapeCanvas::new()
    .rounded_polygon(|| from_svg(path).expect("bad svg"))
    .build()
```

Keep both paths in the same coordinate style (`0..1` or pixels), or the shape will change size on the way. Morphing between paths with different corner counts is not covered by the tests yet.
