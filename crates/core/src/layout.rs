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

impl Default for Card {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub children: Vec<Object>,
}

impl Default for Row {
    fn default() -> Self {
        Self { children: vec![] }
    }
}
