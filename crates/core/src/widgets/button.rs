#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{Button as MasonryButton, Flex};

use crate::elements::Element;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct Button {
    pub text: &'static str,
}

impl Default for Button {
    fn default() -> Self {
        Self { text: "" }
    }
}

impl Button {
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = Box::leak(text.into().into_boxed_str());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        column = column.with_fixed(NewWidget::new(MasonryButton::with_text(self.text)));
        NewWidget::new(column)
    }
}

impl From<Button> for Element {
    fn from(button: Button) -> Self {
        Element::Button(button)
    }
}

impl crate::traits::IntoObject for Button {
    fn into_object(self) -> Object {
        Object::from(Element::from(self))
    }
}
