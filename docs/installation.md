## Installation

Actually, no installation required! Just clone this repo inside your amane dots and you're good to go:

```sh
git clone https://github.com/AstroJr0/rounded_polygon_amane
```

Then declare it in your `main.rs`:

```rust
mod rounded_polygon_amane;
```

This is where it goes:

```
my-shell/
├── Cargo.toml
└── src/
    ├── main.rs                  ← mod rounded_polygon_amane;
    └── rounded_polygon_amane/   ← this repo
        ├── mod.rs
        └── ...
```

No `Cargo.toml` changes. It only uses `amane`, which your shell *supposed to have*.
