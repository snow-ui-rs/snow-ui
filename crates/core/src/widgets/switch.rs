#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::Flex;

use crate::elements::Element;
use crate::object::Object;

#[derive(Debug, Clone, Default)]
pub struct Switch {
    pub children: Vec<Object>,
    pub active: usize,
}

impl Switch {
    pub fn switch_to(&mut self, idx: usize) {
        self.active = if self.children.is_empty() {
            0
        } else {
            idx.min(self.children.len() - 1)
        };
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        if let Some(child) = self
            .children
            .get(self.active.min(self.children.len().saturating_sub(1)))
        {
            column = column.with_fixed(child.into_masonry_widget());
        }
        NewWidget::new(column)
    }
}

impl From<Switch> for Element {
    fn from(switch: Switch) -> Self {
        Element::Switch(switch)
    }
}

impl crate::traits::IntoObject for Switch {
    fn into_object(self) -> Object {
        Object::from(Element::from(self))
    }
}
