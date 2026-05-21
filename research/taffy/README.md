# Taffy Research Examples

This directory contains research examples for `taffy` and is organized for multiple sample programs.

Structure:
- `examples/`: example binaries for taffy experiments
- `src/`: shared code and helper utilities used by examples

Run the minimal example:

```bash
cd research/taffy
cargo run --example minimal
```

The minimal example creates a layout tree with a container and two children, computes layout, and prints the resulting positions and sizes.
