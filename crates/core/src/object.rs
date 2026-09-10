#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::layout::Length;
#[cfg(not(target_arch = "wasm32"))]
use masonry::peniko::{ImageAlphaType, ImageData, ImageFormat};
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{Flex, Image, Label, SizedBox};

use crate::elements::{Element, Text, TextClock};
use crate::layout::{Board, Card, Row};
use crate::traits::IntoObject;
use crate::widgets::Girl;

// ── Object enum ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum Object {
    Board(Board),
    Card(Card),
    Row(Row),
    Element(Element),
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Board(v) => f.debug_tuple("Board").field(v).finish(),
            Self::Card(v) => f.debug_tuple("Card").field(v).finish(),
            Self::Row(v) => f.debug_tuple("Row").field(v).finish(),
            Self::Element(v) => f.debug_tuple("Element").field(v).finish(),
        }
    }
}

impl Object {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        match self {
            Object::Board(board) => board.into_masonry_widget(),
            Object::Card(card) => card.into_masonry_widget(),
            Object::Row(row) => row.into_masonry_widget(),
            Object::Element(Element::Girl(_girl)) => {
                let rgba = image::load_from_memory(include_bytes!("../../../assets/girl.png"))
                    .expect("failed to decode Girl image")
                    .to_rgba8();
                let (width, height) = rgba.dimensions();
                let image_data = ImageData {
                    data: rgba.into_raw().into(),
                    format: ImageFormat::Rgba8,
                    alpha_type: ImageAlphaType::Alpha,
                    width,
                    height,
                };
                let image = NewWidget::new(Image::new(image_data).with_alt_text("Girl"));
                let image =
                    SizedBox::new(image).size(Length::const_px(200.0), Length::const_px(300.0));
                let mut column = Flex::column();
                column = column.with_fixed(NewWidget::new(image));
                NewWidget::new(column)
            }
            Object::Element(Element::Text(text)) => text.into_masonry_widget(),
            Object::Element(Element::TextClock(text_clock)) => {
                let mut column = Flex::column();
                column = column.with_fixed(NewWidget::new(Label::new(
                    text_clock.visible_text().as_str(),
                )));
                NewWidget::new(column)
            }
            Object::Element(Element::Button(button)) => button.into_masonry_widget(),
            Object::Element(Element::Form(form)) => form.clone().into_masonry_widget(),
            Object::Element(Element::TextInput(text_input)) => text_input.into_masonry_widget(),
            Object::Element(Element::Switch(switch_)) => switch_.into_masonry_widget(),
        }
    }

    pub fn update_text_recursive(&mut self, text: impl Into<String>) {
        let next = text.into();
        match self {
            Object::Board(board) => {
                for child in &mut board.children {
                    child.update_text_recursive(next.clone());
                }
            }
            Object::Card(card) => {
                for child in &mut card.children {
                    child.update_text_recursive(next.clone());
                }
            }
            Object::Row(row) => {
                for child in &mut row.children {
                    child.update_text_recursive(next.clone());
                }
            }
            Object::Element(Element::Text(node)) => node.set_text(next.clone()),
            Object::Element(Element::Button(_)) => {}
            Object::Element(Element::Form(form)) => {
                for child in &mut form.children {
                    child.update_text_recursive(next.clone());
                }
            }
            Object::Element(Element::TextInput(_)) => {}
            Object::Element(Element::Switch(switch_)) => {
                for child in &mut switch_.children {
                    child.update_text_recursive(next.clone());
                }
            }
            Object::Element(Element::TextClock(_)) => {}
            Object::Element(Element::Girl(_)) => {}
        }
    }

    pub fn update_button_text_recursive(&mut self, text: impl Into<String>) {
        let next = text.into();
        match self {
            Object::Board(board) => {
                for child in &mut board.children {
                    child.update_button_text_recursive(next.clone());
                }
            }
            Object::Card(card) => {
                for child in &mut card.children {
                    child.update_button_text_recursive(next.clone());
                }
            }
            Object::Row(row) => {
                for child in &mut row.children {
                    child.update_button_text_recursive(next.clone());
                }
            }
            Object::Element(Element::Button(node)) => node.set_text(next.clone()),
            Object::Element(Element::Form(form)) => {
                form.submit_button.set_text(next.clone());
                form.reset_button.set_text(next.clone());
                for child in &mut form.children {
                    child.update_button_text_recursive(next.clone());
                }
            }
            Object::Element(Element::Switch(switch_)) => {
                for child in &mut switch_.children {
                    child.update_button_text_recursive(next.clone());
                }
            }
            Object::Element(Element::Text(_) | Element::TextInput(_) | Element::TextClock(_)) => {}
            Object::Element(Element::Girl(_)) => {}
        }
    }

    pub fn apply_update(&mut self, update: &crate::backend::SnowUpdate) {
        match update {
            crate::backend::SnowUpdate::SetText { text, .. } => {
                self.update_text_recursive(text.clone())
            }
            crate::backend::SnowUpdate::SetButtonText { text, .. } => {
                self.update_button_text_recursive(text.clone())
            }
        }
    }

    pub fn apply_message(&mut self, message: &crate::backend::SnowMessage) {
        for update in message.to_updates() {
            self.apply_update(&update);
        }
    }
}

// ── From impls ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct World {
    pub root: Object,
}

impl Object {
    fn collect_button_ids(&self, next_id: &mut u64, ids: &mut Vec<u64>) {
        match self {
            Object::Board(board) => {
                for child in &board.children {
                    child.collect_button_ids(next_id, ids);
                }
            }
            Object::Card(card) => {
                for child in &card.children {
                    child.collect_button_ids(next_id, ids);
                }
            }
            Object::Row(row) => {
                for child in &row.children {
                    child.collect_button_ids(next_id, ids);
                }
            }
            Object::Element(Element::Button(_)) => {
                ids.push(*next_id);
                *next_id += 1;
            }
            Object::Element(Element::Form(form)) => {
                for child in &form.children {
                    child.collect_button_ids(next_id, ids);
                }
            }
            Object::Element(Element::Switch(switch_)) => {
                for child in &switch_.children {
                    child.collect_button_ids(next_id, ids);
                }
            }
            Object::Element(Element::Text(_))
            | Object::Element(Element::TextClock(_))
            | Object::Element(Element::TextInput(_))
            | Object::Element(Element::Girl(_)) => {}
        }
    }
}

impl World {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        self.root.into_masonry_widget()
    }

    pub fn button_ids(&self) -> Vec<u64> {
        let mut ids = Vec::new();
        let mut next_id = 2u64;
        self.root.collect_button_ids(&mut next_id, &mut ids);
        ids
    }

    pub fn apply_update(&mut self, update: &crate::backend::SnowUpdate) {
        self.root.apply_update(update);
    }

    pub fn apply_message(&mut self, message: &crate::backend::SnowMessage) {
        self.root.apply_message(message);
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            root: Object::Board(Board::default()),
        }
    }
}

impl From<Board> for Object {
    fn from(b: Board) -> Self {
        Object::Board(b)
    }
}

impl From<Girl> for Object {
    fn from(g: Girl) -> Self {
        Object::from(Element::from(g))
    }
}

impl From<Card> for Object {
    fn from(c: Card) -> Self {
        Object::Card(c)
    }
}

impl From<Row> for Object {
    fn from(r: Row) -> Self {
        Object::Row(r)
    }
}

impl From<Element> for Object {
    fn from(e: Element) -> Self {
        Object::Element(e)
    }
}

impl From<Text> for Object {
    fn from(t: Text) -> Self {
        // Convert Text -> Element (via `From<Text> for Element`) and wrap into Object::Element
        Object::Element(t.into())
    }
}

impl From<TextClock> for Object {
    fn from(t: TextClock) -> Self {
        // Convert TextClock -> Element and wrap into Object::Element
        Object::Element(t.into())
    }
}

impl From<u128> for Object {
    fn from(n: u128) -> Self {
        // Convert number to a textual representation for demonstration.
        let s = format!("{}", n);
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        Text {
            text: leaked,
            ..Text::default()
        }
        .into()
    }
}

impl<T: IntoObject> From<T> for Object {
    fn from(t: T) -> Self {
        t.into_object()
    }
}
