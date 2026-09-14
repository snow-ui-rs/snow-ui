#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::Flex;

use crate::elements::Element;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct Row {
    pub children: Vec<Object>,
}

impl Row {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut row = Flex::row();
        for child in &self.children {
            row = row.with_fixed(child.into_masonry_widget());
        }
        NewWidget::new(row)
    }
}

impl Default for Row {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl From<Row> for Element {
    fn from(row: Row) -> Self {
        Element::Row(row)
    }
}
