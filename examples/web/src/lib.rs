#[path = "../../click.rs"]
#[cfg(target_arch = "wasm32")]
mod click;
#[path = "../../clock.rs"]
#[cfg(target_arch = "wasm32")]
mod clock;
#[path = "../../login.rs"]
#[cfg(target_arch = "wasm32")]
mod login;
#[path = "../../lovely_girl.rs"]
#[cfg(target_arch = "wasm32")]
mod lovely_girl;
#[path = "../../timer.rs"]
#[cfg(target_arch = "wasm32")]
mod timer;

#[cfg(target_arch = "wasm32")]
use snow_ui::{Board, Object, World, launch};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn world() -> World {
    let click = click::world();
    let clock = clock::world();
    let login = login::world();
    let lovely_girl = lovely_girl::world();
    let timer = timer::world();

    World {
        root: Object::from(Board {
            children: vec![
                click.root,
                clock.root,
                login.root,
                lovely_girl.root,
                timer.root,
            ],
            ..Default::default()
        }),
    }
}

#[wasm_bindgen(start)]
#[cfg(target_arch = "wasm32")]
pub fn start() {
    launch(world);
}