# snow-ui-flake

`snow-ui-flake` is the native implementation foundation for Snow UI on desktop and mobile platforms. It brings together the libraries intended for Snow UI's native backend:

- [Taffy](https://github.com/DioxusLabs/taffy) for layout calculation
- [winit](https://github.com/rust-windowing/winit) for windows and input events
- [wgpu](https://github.com/gfx-rs/wgpu) for GPU rendering

## Current status

The crate currently re-exports these dependencies so native backend work can be developed behind a dedicated crate boundary. Snow UI-specific layout, windowing, and rendering abstractions have not been implemented here yet.
