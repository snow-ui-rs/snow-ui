//! A simple timer example that implements its own ticker.

use snow_ui::prelude::*;
use tokio::time::Duration;

#[message]
struct SimpleTextTimerTickEvent {}

#[element]
struct SimpleTextTimer {
    seconds: State<u128>,
    timer: IntervalTimer<SimpleTextTimerTickEvent>,
}

register_handler!(
    impl MessageHandler<SimpleTextTimerTickEvent> for SimpleTextTimer {
        async fn handle(&mut self, _: &SimpleTextTimerTickEvent, _: &mut MessageContext) {
            self.seconds.update(|s| *s += 1);
        }
    }
);

fn simple_text_timer() -> Object {
    obj!(SimpleTextTimer {
        seconds: State::new(0),
        timer: IntervalTimer::from_interval(Duration::from_secs(1)),
    })
}

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
                            text: "Timer Example ⏱️",
                        },],
                    },
                    Row {
                        children: list![simple_text_timer()],
                    },
                ],
            },],
        }),
        ..default()
    }
}

fn main() {
    snow_ui::launch(world);
}
