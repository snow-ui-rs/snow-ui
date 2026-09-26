#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::layout::Dim;
#[cfg(not(target_arch = "wasm32"))]
use masonry::properties::Dimensions;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::Flex;

use crate::elements::Element;
use crate::object::Object;

#[derive(Debug, Clone, Default)]
pub struct Card {
    pub children: Vec<Object>,
}

impl Card {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        for child in &self.children {
            column = crate::object::add_masonry_child(column, child, child.into_masonry_widget());
        }
        NewWidget::new(column).with_props(Dimensions::width(Dim::MaxContent))
    }
}

impl From<Card> for Element {
    fn from(card: Card) -> Self {
        Element::Card(card)
    }
}
