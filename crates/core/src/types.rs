#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Size {
    ViewportWidth,
    ViewportHeight,
}

pub const VIEWPORT_WIDTH: Size = Size::ViewportWidth;
pub const VIEWPORT_HEIGHT: Size = Size::ViewportHeight;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}
