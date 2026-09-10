# Snow UI Web Example

This page provides browser versions of the five Rust examples: click, clock,
login, lovely girl, and timer. It exercises the `web-sys` renderer on
`wasm32-unknown-unknown`.

Build the WASM module and generate browser bindings from the workspace root:

```bash
cargo build \
  --manifest-path Cargo.toml \
  --package snow-ui-web-example \
  --target wasm32-unknown-unknown

wasm-bindgen \
  target/wasm32-unknown-unknown/debug/snow_ui_web_example.wasm \
  --out-dir examples/web/pkg \
  --target web
```

Serve the example over HTTP:

```bash
python3 -m http.server 8080 --directory .
```

Open <http://127.0.0.1:8080/examples/web/> and verify the five example sections in
`#snow-root`. The click and login buttons update their sections, while the
clock and timer update once per second.