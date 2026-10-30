# Placing a Shape in a Window

`ShapeCanvas::build()` returns an amane `Rectangle`. It is therefore placed in an interface in the same way as any other widget.

## In a Window

```rust
LayerWindow::new().width(96.0).height(96.0).child(heart())
```

Here `heart()` is the function from [Defining a Material Shape](./defm3shape.md). The window is given the same size as the canvas, so that the shape fills it exactly.

## Content on Top of a Shape

To place content over a shape, both are put into a `Stack`. A stack draws its children in order, so the shape is listed first and the content, for example a `Text`, after it.

## Several Canvases

The state of a morph is stored under the canvas's `id`, which by default is derived from the place in the source code where `ShapeCanvas::new()` is called. Canvases created in a loop, or shared by several windows, would therefore share one state. Each of them should be given its own name with `.id(...)`:

```rust
ShapeCanvas::new().id(format!("shape-{i}"))
```
