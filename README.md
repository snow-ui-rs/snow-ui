# Snow UI: Pure Rust UI Framework

## Philosophy

The UI is a projection of a virtual world onto the screen.


## Example: A lovely girl

A minimal component example demonstrating the `#[snow]` attribute  and constructing a component with several fields.

```rust
use snow_ui::prelude::*;

#[snow]
struct LovelyGirl {
    girl: Girl,
}

fn lovely_girl() -> Object {
    obj!(LovelyGirl {
        girl: Girl {
            hair_color: HairColor::Black,
            skin_color: SkinColor::Yellow,
            body_type: BodyType::Slim,
            appearance: Appearance::Beautiful,
            every_morning: vec![GirlActions::SayHi, GirlActions::PrepareBreakfast],
        },
    })
}

fn world() -> World {
    World {
        root: lovely_girl(),
    }
}

fn main() {
    snow_ui::launch(world);
}
```

---

## Example: Text Clock

A simple board example: a `Board` containing a `Card` with two rows (`Text` and `TextClock`).

```rust
use snow_ui::prelude::*;

fn world() -> World {
    World {
        root: obj!(Board {
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
            children: list![Card {
                children: list![
                    Row {
                        children: list![Text {
                            text: "Carpe diem 🎉",
                        },],
                    },
                    Row {
                        children: list![TextClock { format: "%H:%M:%S" },],
                    },
                ],
            },],
        }),
    }
}

fn main() {
    snow_ui::launch(world);
}
```
