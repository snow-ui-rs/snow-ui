#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::Flex;

use crate::object::Object;
use crate::types::{HAlign, Size, VAlign, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

#[derive(Debug, Clone)]
pub struct Board {
    pub width: Size,
    pub height: Size,
    pub h_align: HAlign,
    pub v_align: VAlign,
    pub children: Vec<Object>,
}

impl Board {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        for child in &self.children {
            column = column.with_fixed(child.into_masonry_widget());
        }
        NewWidget::new(column)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
            children: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub children: Vec<Object>,
}

impl Card {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        for child in &self.children {
            column = column.with_fixed(child.into_masonry_widget());
        }
        NewWidget::new(column)
    }
}

impl Default for Card {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

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
