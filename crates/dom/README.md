# snow-ui-dom

`snow-ui-dom` is the Web implementation foundation for Snow UI. It provides the browser DOM layer used to build Snow UI interfaces on the Web, using `wasm-bindgen` and `web-sys` on the `wasm32` target.

## Available helpers

- `window()` and `document()` access the current browser window and document.
- `create_element()` creates a DOM element.
- `append()` adds an element to a parent.
- `clear()` removes all child nodes from an element.

The crate currently provides these low-level DOM helpers; a complete Snow UI DOM renderer and event integration are not part of this crate yet.
