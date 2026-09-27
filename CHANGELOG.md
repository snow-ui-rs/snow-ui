# Changelog

## [0.0.1] - 2026-09-27

### Added
- `obj!` and `list!` macros for building UI object trees
- `#[element]`, `#[message]`, and `IntoObject` procedural macros
- `State<T>` as a basic observable state primitive
- Cloneable, typed `World.data` storage with lazy default initialization
- Use `masonry` as native backend framework, while use `DOM` on web
- Basic UI elements and containers, including `Board`, `Card`, `Row`, `Text`, `Button`, and `Switch`
- Form support with labeled text inputs, JSON serialization, submit handlers, and reset buttons
- Async click and message handlers, plus switchable views
- Built-in text clock and interval timer support
- Five examples: a lovely girl, a text clock, a custom timer, a click counter, and a login form
- An initial typed message bus

### Known Limitations
- The API is unstable
- The web backend is incomplete
- No comprehensive layout, accessibility, or IME strategy
- `ServerApi` is currently a stub and does not perform real network requests