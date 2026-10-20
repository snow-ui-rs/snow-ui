# snow-ui-flake

`snow-ui-flake` is the native implementation foundation for Snow UI on desktop and mobile platforms. It brings together the libraries intended for Snow UI's native backend:

- [Taffy](https://github.com/DioxusLabs/taffy) for layout calculation
- [winit](https://github.com/rust-windowing/winit) for windows and input events
- [wgpu](https://github.com/gfx-rs/wgpu) for GPU rendering

## Current status

The crate re-exports these dependencies behind a dedicated native backend boundary. It also defines platform-neutral native input events and a coalescing refresh handle, so events and local invalidations can be passed across the backend boundary without adopting Masonry's widget or driver model.

`RefreshHandle` tracks layout and paint invalidations by stable `WidgetId`. Repeated invalidations are coalesced. A layout invalidation can include the changed widget's path to the root, while directly marking only the changed widget for paint; the layout pass can then identify other widgets affected by changed geometry. A full invalidation supersedes pending local work.

`LayoutTree` maps a platform-neutral `LayoutNode` tree to Taffy nodes, computes viewport layout, and returns geometry by `WidgetId`. Style changes and refresh batches mark Taffy nodes dirty so it can reuse cached layout for unaffected nodes.

`PaintScene` retains solid-rectangle and text nodes. It tracks old and new bounds when nodes change, merges overlapping damage regions, and produces a `PaintFrame` containing affected draw commands in z-order. `WgpuRenderer` uses wgpu for shapes and glyphon for text, with scissor rectangles for local damage and logical-coordinate scaling for high-DPI targets.

`NativeSurface` creates and configures a wgpu surface for a winit window, keeps a persistent offscreen target for partial updates, and presents the cached image. `WinitEventAdapter` translates platform input into `NativeEvent` and can dispatch it through an application-provided `NativeEventSink`. Snow UI can adapt that sink to its EventBus without making flake depend on core message types. EventBus handlers can use a cloned `NativeRefreshHandle` to queue layout/paint invalidations and wake the owning window; redraw requests are coalesced per window. The application's `user_event` handler must call `RedrawProxy::redraw_dispatched` for each `NativeUserEvent::Redraw` before requesting the actual window redraw, allowing later invalidations to wake it again. Snow UI owns the `EventLoop` and `ApplicationHandler`, and should drop/recreate `NativeSurface` across suspend/resume. Image paint primitives are not implemented yet.

Run the native rectangle, text, and local-refresh example with `cargo run -p snow-ui-flake --example minimal`.
