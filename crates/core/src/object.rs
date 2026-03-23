use crate::elements::{Element, Text, TextClock};
use crate::girl::Girl;
use crate::layout::{Board, Card, Row};
use crate::traits::IntoObject;

#[allow(dead_code)]
#[derive(Debug)]
pub struct World {
    pub root: Object,
}

impl Default for World {
    fn default() -> Self {
        Self {
            root: Object::Board(Board::default()),
        }
    }
}

// ── Object enum ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Object {
    Board(Board),
    Girl(Girl),
    Card(Card),
    Row(Row),
    Element(Element),
}

// ── From impls ───────────────────────────────────────────────────────────────

impl From<Board> for Object {
    fn from(b: Board) -> Self {
        Object::Board(b)
    }
}

impl From<Girl> for Object {
    fn from(g: Girl) -> Self {
        Object::Girl(g)
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
        Text { text: leaked }.into()
    }
}

impl<T: IntoObject> From<T> for Object {
    fn from(t: T) -> Self {
        t.into_object()
    }
}
