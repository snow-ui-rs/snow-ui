//! A minimal example demonstrating a virtual world and a typical element with properties and actions.

use snow_ui::prelude::*;

#[element]
struct LovelyGirl {
    girl: Girl,
}

impl LovelyGirl {
    pub fn obj() -> Object {
        obj!(LovelyGirl {
            girl: Girl {
                hair_color: HairColor::Black,
                skin_color: SkinColor::Yellow,
                body_type: BodyType::Slim,
                appearance: Appearance::Beautiful,
                every_morning: actions![GirlActions::SayHi, GirlActions::PrepareBreakfast,],
            },
        })
    }
}

pub fn world() -> World {
    World {
        root: LovelyGirl::obj(),
        ..default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    snow_ui::launch(world);
}
