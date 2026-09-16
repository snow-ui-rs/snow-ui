#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::layout::Length;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{Flex, Label, SizedBox, TextInput as MasonryTextInput};

use crate::elements::Element;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct TextInput {
    pub label: &'static str,
    pub name: &'static str,
    pub r#type: &'static str,
    pub max_len: u32,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            label: "",
            name: "",
            r#type: "text",
            max_len: 0,
        }
    }
}

impl TextInput {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut row = Flex::row();
        if !self.label.is_empty() {
            row = row.with_fixed(NewWidget::new(Label::new(self.label)));
        }
        let input_width = self.name.chars().count() as f64 * 12.0 + 24.0;
        let input = MasonryTextInput::new("").with_placeholder(self.name);
        row = row.with_fixed(NewWidget::new(
            SizedBox::new(NewWidget::new(input)).width(Length::const_px(input_width)),
        ));
        NewWidget::new(row)
    }
}

impl From<TextInput> for Element {
    fn from(input: TextInput) -> Self {
        Element::TextInput(input)
    }
}

impl crate::traits::IntoObject for TextInput {
    fn into_object(self) -> Object {
        Object::from(Element::from(self))
    }
}
