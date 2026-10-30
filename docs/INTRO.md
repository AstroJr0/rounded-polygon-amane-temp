# Introduction

`rounded_polygon_amane` is a shape library written specifically for [amane](https://github.com/MystiaFin/amane), a desktop UI framework for Rust. It draws polygons whose corners are rounded, and it animates a smooth transition (a *morph*) from one polygon to another.

## Concepts

The library is built on a small number of concepts. Each page of this documentation relies on them.

**Corner.** A corner is a point of a polygon together with a description of how it is rounded. In code it is a `PointNRound`: an `Offset` (the position) paired with a `CornerRounding` (the rounding).

**Rounding.** A `CornerRounding` consists of a radius and an optional smoothing value. The radius states how far the corner is cut away. The smoothing value, between `0.0` and `1.0`, softens the curve where it meets the straight edges. A radius of zero produces a sharp corner.

**Polygon.** A `RoundedPolygon` is the finished outline: a closed sequence of straight edges and rounded corners, stored internally as cubic Bézier curves. A polygon can be obtained in three ways:

1. from the thirty-five ready-made Material Design shapes ([Defining a Material Shape](./defm3shape.md));
2. from a list of corners ([Defining a Custom Polygon](./def_custom_polygon.md));
3. from an SVG path string ([Using SVG Paths](./using_svgs.md)).

All three produce the same type. Consequently, they may be used interchangeably, including as the two ends of a morph.

**Canvas.** A `ShapeCanvas` is the widget that draws a polygon. It is configured through a chain of methods (colour, size, border, and so on) and finished with `.build()`, which returns an ordinary amane `Rectangle`.

**Morph.** A morph is the animated transition between two polygons. The canvas performs it automatically.

## How a Morph Happens

An amane view is rebuilt every time the interface is redrawn. A `ShapeCanvas` therefore cannot hold an animation inside itself; the state of the animation is kept in a store outside the view, and is identified by the canvas's `id`. By default, the `id` is derived from the place in the source code where `ShapeCanvas::new()` is called.

On every redraw, the canvas compares the polygon it has just received with the polygon it received previously. If the two are identical, nothing happens. If they differ, the canvas starts a morph: it pairs the features (corners and edges) of the old polygon with those of the new polygon, and then moves each pair from its old shape to its new shape over the configured duration. The pairing is computed by the library, so two polygons need not have the same number of corners.

The consequence is that the programmer never requests a morph explicitly. A morph is obtained by changing which polygon is handed to the canvas, typically in response to a change of state such as a click.

## Coordinates

Unless stated otherwise, a polygon is described in a unit square: both coordinates range from `0.0` to `1.0`, and the y-axis grows downwards. The canvas scales this square to the size given by `.size(width, height)`. Polygons that are already expressed in pixels are drawn with `.normalized(false)`.

## Reading This Documentation

The pages are intended to be read in the order of the table of contents. The [Cheat Sheet](./CHEATSHEET.md) lists every function on a single page, and is intended as a reference once the concepts above are familiar.

## Credits and Licence

This library is an amane port of [Androidx's shape library](https://developer.android.com/jetpack/androidx/releases/graphics), also [Knugel's TypeScript port](https://github.com/Knugel/rounded-polygon-ts) and [end-4's QML port](https://github.com/end-4/rounded-polygon-qmljs). Note that is distributed under the [Apache-2.0](https://github.com/AstroJr0/rounded_polygon_amane/blob/main/LICENSE) licence, similar to the projects above.
