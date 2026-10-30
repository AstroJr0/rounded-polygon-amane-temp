# Defining a Custom Polygon

A custom polygon is described by a list of corners, one *slice* of the shape, together with the number of times the slice is repeated around a centre point.

## Corners

A `PointNRound` is a corner. It states where the corner lies and how it is rounded. Two constructors exist:

- `PointNRound::unrounded(offset)` creates a sharp corner. Can be used for making pixalated shapes.
- `PointNRound::new(offset, rounding)` creates a rounded corner.

A rounding is created with `CornerRounding::from_radius(radius)`, or with `CornerRounding::new(radius, smoothing)` when a smoothing value between `0.0` and `1.0` is also wanted. The radius is relative to the size of the polygon.

## Example

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

## Explanation

`custom_polygon(points, reps, center, mirroring)` takes four arguments:

| Argument | Elaboration of the Argument |
|---|---|
| `points` | the corners of one slice |
| `reps` | how many times the slice is repeated |
| `center` | the point around which the slice is rotated |
| `mirroring` | when `true`, every other copy is mirrored |

Inside the example, the slice contains two corners: 
1. A sharp tip at the top of the unit square
2. And a rounded dip beside it. The slice is repeated five times around the centre `(0.5, 0.5)`.

Each copy is rotated by *(360deg / 5 = )*  72deg relative to the previous one, so the ten corners together form a five-pointed star.

The call to `.normalized()` scales and moves the finished polygon to fit the unit square, keeping its proportions. It is not required when the corners already lie within the **unit square**, but it guarantees that the shape fills the canvas.

> [!TIP]
> Unit Square is basically a square with and height are `1.0`. Shapes inside Unit Square can be used to scale up a shape without losing its design *(not quality bcz everything here is a vector)*

When `mirroring` is `true`, each second copy is reflected instead of merely rotated. This produces symmetric shapes, such as a petal or a flower, from a single slice.
