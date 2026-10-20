# snow-ui-flake

`snow-ui-flake` is the native implementation foundation for Snow UI on desktop and mobile platforms. It brings together the libraries intended for Snow UI's native backend:

- [Taffy](https://github.com/DioxusLabs/taffy) for layout calculation
- [winit](https://github.com/rust-windowing/winit) for windows and input events
- [wgpu](https://github.com/gfx-rs/wgpu) for GPU rendering

## Current status

The crate re-exports these dependencies behind a dedicated native backend boundary. It also defines platform-neutral native input events and a coalescing refresh handle, so events and local invalidations can be passed across the backend boundary without adopting Masonry's widget or driver model.

`RefreshHandle` tracks layout and paint invalidations by stable `WidgetId`. Repeated invalidations are coalesced. A layout invalidation can include the changed widget's path to the root, while directly marking only the changed widget for paint; the layout pass can then identify other widgets affected by changed geometry. A full invalidation supersedes pending local work.

`LayoutTree` maps a platform-neutral `LayoutNode` tree to Taffy nodes, computes viewport layout, and returns geometry by `WidgetId`. Style changes and refresh batches mark Taffy nodes dirty so it can reuse cached layout for unaffected nodes. Window redraw scheduling and incremental wgpu painting are not implemented yet.
