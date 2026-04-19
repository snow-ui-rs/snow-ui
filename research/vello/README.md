# Vello Research Examples

This directory contains research examples for `vello` and is organized for multiple sample programs.

Structure:
- `examples/`: multiple example binaries for different vello experiments
- `src/`: shared code and helper utilities used by examples

Run the minimal example:

```bash
cd research/vello
cargo run --example minimal
```

Run the render example:

```bash
cd research/vello
cargo run --example render
```

The `render` example creates a real Vello scene, renders it to an intermediate texture, and presents it in a Winit window.
