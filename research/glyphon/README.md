# Glyphon Research Examples

This directory contains research examples for `glyphon` and is organized for multiple sample programs.

Structure:
- `examples/`: example binaries for glyphon experiments
- `src/`: shared code and helper utilities used by examples

Run the minimal example:

```bash
cd research/glyphon
cargo run --example minimal
```

The minimal example creates a `wgpu` window, initializes `glyphon` text rendering, and displays a simple text layout.
