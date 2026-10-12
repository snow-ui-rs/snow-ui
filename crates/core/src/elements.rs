pub use crate::widgets::{
    Appearance, Board, BodyType, Button, Card, Form, Girl, GirlActions, HairColor, IntervalTimer,
    Row, SkinColor, SubmitHandler, Switch, Text, TextClock, TextInput,
};

#[derive(Debug, Clone)]
pub enum Element {
    Board(Board),
    Card(Card),
    Row(Row),
    Text(Text),
    TextClock(TextClock),
    Button(Button),
    Form(Form),
    TextInput(TextInput),
    Switch(Switch),
    Girl(Girl),
}

impl Element {
    /// Return the stable identity assigned to this rendered widget.
    pub fn id(&self) -> crate::types::WidgetId {
        match self {
            Self::Board(widget) => widget.id,
            Self::Card(widget) => widget.id,
            Self::Row(widget) => widget.id,
            Self::Text(widget) => widget.id,
            Self::TextClock(widget) => widget.id,
            Self::Button(widget) => widget.id,
            Self::Form(widget) => widget.id,
            Self::TextInput(widget) => widget.id,
            Self::Switch(widget) => widget.id,
            Self::Girl(widget) => widget.id,
        }
    }
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
