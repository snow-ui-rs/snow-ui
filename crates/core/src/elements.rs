pub use crate::widgets::{Button, IntervalTimer, Switch, Text, TextClock, TextInput};

use crate::form::Form;

#[derive(Debug, Clone)]
pub enum Element {
    Text(Text),
    TextClock(TextClock),
    Button(Button),
    Form(Form),
    TextInput(TextInput),
    Switch(Switch),
}

#[cfg(test)]
mod tests {
    use super::Text;
    use crate::state::State;

    #[test]
    fn text_uses_bound_state_as_visible_text() {
        let state = State::new(1u128);
        let text = Text::from_state(&state);

        assert_eq!(text.visible_text(), "1");
        state.set(2);
        assert_eq!(text.visible_text(), "2");
    }

    #[test]
    fn text_without_state_uses_static_text() {
        let text = Text {
            text: "hello",
            ..Text::default()
        };

        assert_eq!(text.visible_text(), "hello");
    }
}
