//! A minimal example demonstrating basic usage of framework-provided components (uses the built-in `Girl` component).

use snow_ui::prelude::*;

#[element]
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
            every_morning: vec![GirlActions::SayHi, GirlActions::PrepareBreakfast,],
        },
    })
}

fn world() -> World {
    World {
        root: lovely_girl(),
        ..default()
    }
}

fn main() {
    snow_ui::launch(world);
}
