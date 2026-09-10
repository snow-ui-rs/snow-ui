#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{Flex, Label};

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
        let mut column = Flex::column();
        if !self.label.is_empty() {
            column = column.with_fixed(NewWidget::new(Label::new(self.label)));
        }
        column = column.with_fixed(NewWidget::new(Label::new(self.name)));
        NewWidget::new(column)
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
