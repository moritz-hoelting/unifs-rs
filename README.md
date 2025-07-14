# UniFs: A Unified Filesystem Abstraction in Rust

Abstraction for the `std::fs` module as a trait in Rust.

## Trait implementations

- `PhysicalFs`: Unit struct representing the physical root filesystem.
- `ReadonlyFs`: Wrapper around the `UniFs` trait that provides a read-only view of the filesystem.

## Usage

To use the UniFs library, add it to your `Cargo.toml`:

```toml
[dependencies]
unifs = "0.1"
```

## Roadmap

- In-memory filesystem
- Stacked filesystems