// Minimal mock of an `snow_ui` crate used by the example in `main.rs`.
// Provides the types and items referenced by the example so the crate builds.

pub mod prelude {
    pub use super::{
        Board, Card, CENTER, MIDDLE, Row, Text, TextTimer, VIEWPORT_HEIGHT, VIEWPORT_WIDTH, World,
        Widget, IntoWidget, Girl, HairColor, SkinColor, BodyType, Appearance, GirlActions,
        HAlign, VAlign,
    };

    // Re-export the derive macro so examples can `use snow_ui::prelude::*` and write
    // `#[derive(IntoWidget)]` without importing `snow_ui_macros` explicitly.
    pub use snow_ui_macros::IntoWidget;

    /// Helper to allow `..default()` shorthand in user code (like Bevy's prelude).
    ///
    /// Example: `Row { ..default() }`
    #[allow(dead_code)]
    pub fn default<T: Default>() -> T {
        T::default()
    }
} 

/// Launch the UI using a builder function that returns a `World`.
///
/// Example: `snow_ui::launch(world);` where `fn world() -> World { ... }`.
pub fn launch<F: FnOnce() -> World>(builder: F) {
    let world = builder();
    println!("Launching snow_ui with world:\n{:#?}", world);
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct World {
    pub root: Widget,
}

impl Default for World {
    fn default() -> Self {
        Self {
            root: Widget::Board(Board::default()),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Board {
    pub width: Size,
    pub height: Size,
    pub h_align: HAlign,
    pub v_align: VAlign,
    pub children: Vec<Card>,
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct Card {
    pub children: Vec<Row>,
}

impl Default for Card {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Row {
    pub children: Vec<Element>,
}

impl Default for Row {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Element {
    Text(Text),
    TextTimer(TextTimer),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Text {
    pub text: &'static str,
}

impl Text {
    /// Return an `Element::Text` so the call sites using `Text::from_str` can be placed
    /// directly inside a `Vec` of `Element`.
    pub fn from_str(s: &'static str) -> Element {
        Element::Text(Text { text: s })
    }
}

impl Default for Text {
    fn default() -> Self {
        Self { text: "" }
    }
}

impl From<Text> for Element {
    fn from(t: Text) -> Self {
        Element::Text(t)
    }
} 

#[allow(dead_code)]
#[derive(Debug)]
pub struct TextTimer {
    pub format: &'static str,
}

impl From<TextTimer> for Element {
    fn from(t: TextTimer) -> Self {
        Element::TextTimer(t)
    }
}

impl TextTimer {
    /// Construct a `TextTimer` with the provided format and return it as an `Element`.
    ///
    /// Use `TextTimer::with_format("%H:%M:%S")` to create a timer element.
    pub fn with_format(format: &'static str) -> Element {
        Element::TextTimer(TextTimer { format })
    }
}

impl Default for TextTimer {
    fn default() -> Self {
        Self { format: "" }
    }
}

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

pub const CENTER: HAlign = HAlign::Center;
pub const MIDDLE: VAlign = VAlign::Middle;

// Widget system
#[allow(dead_code)]
#[derive(Debug)]
pub enum Widget {
    Board(Board),
    Girl(Girl),
}

impl From<Board> for Widget {
    fn from(b: Board) -> Self {
        Widget::Board(b)
    }
}

impl From<Girl> for Widget {
    fn from(g: Girl) -> Self {
        Widget::Girl(g)
    }
}

pub trait IntoWidget {
    fn into_widget(self) -> Widget;
}

impl<T: IntoWidget> From<T> for Widget {
    fn from(t: T) -> Self {
        t.into_widget()
    }
}


// Girl component
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Girl {
    pub hair_color: HairColor,
    pub skin_color: SkinColor,
    pub body_type: BodyType,
    pub appearance: Appearance,
    pub every_morning: Vec<GirlActions>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum HairColor {
    Black,
    Brown,
    Blonde,
    Red,
}

impl Default for HairColor {
    fn default() -> Self {
        HairColor::Brown
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum SkinColor {
    Yellow,
    Light,
    Dark,
}

impl Default for SkinColor {
    fn default() -> Self {
        SkinColor::Light
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum BodyType {
    Slim,
    Average,
    Curvy,
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::Average
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Appearance {
    Beautiful,
    Cute,
    Plain,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance::Cute
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum GirlActions {
    SayHi,
    PrepareBreakfast,
}
