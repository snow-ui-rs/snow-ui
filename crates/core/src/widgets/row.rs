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
use crate::types::HAlign;

#[derive(Debug, Clone)]
pub struct Row {
    pub h_align: HAlign,
    pub children: Vec<Object>,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            h_align: HAlign::Left,
            children: vec![],
        }
    }
}

impl Row {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut row = Flex::row();
        for child in &self.children {
            row = row.with_fixed(child.into_masonry_widget());
        }
        NewWidget::new(row).with_props(Dimensions::width(Dim::MaxContent))
    }
}

impl From<Row> for Element {
    fn from(row: Row) -> Self {
        Element::Row(row)
    }
}
