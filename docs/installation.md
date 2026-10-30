<h1 align="center">Installation</h1>

This library is a module, not a whole crate. No usual sense installation is required. The repository is placed inside the source tree of an amane shell, and the shell declares it as a module. And everything would work as it should be.

## Steps

1. Clone the repository into the `src` directory of the shell, so that the folder is named `rounded_polygon_amane`:

   ```sh
   git clone https://github.com/AstroJr0/rounded_polygon_amane
   ```

2. Declare the module in `main.rs`:

   ```rust
   mod rounded_polygon_amane;
   ```

This is how it should look like:

```
my-awesome-shell/
├── Cargo.toml
└── src/
    ├── main.rs                  ← mod rounded_polygon_amane;
    └── rounded_polygon_amane/   ← this repository
        ├── mod.rs
        └── ...
```

## Dependencies

No change to `Cargo.toml` is necessary. The module depends only on `amane`, which every amane shell already provides.

Because the module is part of the shell's own crate, its items are reached through `crate::rounded_polygon_amane`, as the examples in this documentation show.
