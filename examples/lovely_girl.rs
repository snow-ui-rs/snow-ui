//! A minimal example for `snow-ui` demonstrating basic usage.
//!
//! Build and run with: `cargo run --example lovely_girl`

use snow_ui::prelude::*;

#[derive(IntoWidget)]
struct LovelyGirl {
    girl: Girl,
}

fn lovely_girl() -> Widget {
    LovelyGirl {
            girl: Girl {
                hair_color: HairColor::Black,
                skin_color: SkinColor::Yellow,
                body_type: BodyType::Slim,
                appearance: Appearance::Beautiful,
                every_morning: vec![
                    GirlActions::SayHi,
                    GirlActions::PrepareBreakfast,
                ],
                ..default()
            },
    }.into()
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