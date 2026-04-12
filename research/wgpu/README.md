# WGPU Research Examples

This directory contains research examples for `wgpu` and is organized for multiple sample programs.

Structure:
- `examples/`: multiple example binaries for different wgpu experiments
- `src/`: shared code and helper utilities used by examples

Run the minimal example:

```bash
cd research/wgpu
cargo run --example minimal
```

The minimal example initializes a `wgpu` instance, selects a GPU adapter, creates a device and queue, and prints adapter/device information.
