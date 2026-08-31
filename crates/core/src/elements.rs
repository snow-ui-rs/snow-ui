use masonry::core::NewWidget;
use masonry::widgets::{Button as MasonryButton, Flex, Label};

use crate::form::Form;
use crate::object::Object;
use crate::traits::IntoObject;

#[derive(Debug, Clone)]
pub enum Element {
    Text(Text),
    TextClock(TextClock),
    Button(Button),
    Form(Form),
    TextInput(TextInput),
    Switch(Switch),
}

// ── Text ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Text {
    pub text: &'static str,
}

impl Default for Text {
    fn default() -> Self {
        Self { text: "" }
    }
}

impl Text {
    pub fn set_text(&mut self, text: impl Into<String>) {
        let s = text.into();
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        self.text = leaked;
    }

    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        column = column.with_child(NewWidget::new(Label::new(self.text)));
        NewWidget::new(column)
    }
}

impl From<Text> for Element {
    fn from(t: Text) -> Self {
        Element::Text(t)
    }
}

// ── TextClock ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TextClock {
    pub format: &'static str,
}

impl Default for TextClock {
    fn default() -> Self {
        Self { format: "" }
    }
}

impl From<TextClock> for Element {
    fn from(t: TextClock) -> Self {
        Element::TextClock(t)
    }
}

// ── Button ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Button {
    pub text: &'static str,
}

impl Default for Button {
    fn default() -> Self {
        Self { text: "" }
    }
}

impl Button {
    pub fn set_text(&mut self, text: impl Into<String>) {
        let s = text.into();
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        self.text = leaked;
    }

    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        column = column.with_child(NewWidget::new(MasonryButton::with_text(self.text)));
        NewWidget::new(column)
    }
}

impl From<Button> for Element {
    fn from(b: Button) -> Self {
        Element::Button(b)
    }
}

impl IntoObject for Button {
    fn into_object(self) -> Object {
        Element::from(self).into()
    }
}

// ── TextInput ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TextInput {
    /// Optional label text shown next to the input field.
    pub label: &'static str,
    pub name: &'static str,
    pub r#type: &'static str,
    /// Optional maximum length for input. If `0` then no limit is applied.
    pub max_len: u32,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            label: "",
            name: "",
            r#type: "text",
            max_len: 0,
        }
    }
}

impl TextInput {
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        if !self.label.is_empty() {
            column = column.with_child(NewWidget::new(Label::new(self.label)));
        }
        column = column.with_child(NewWidget::new(Label::new(self.name)));
        NewWidget::new(column)
    }
}

impl From<TextInput> for Element {
    fn from(t: TextInput) -> Self {
        Element::TextInput(t)
    }
}

impl IntoObject for TextInput {
    fn into_object(self) -> Object {
        Element::from(self).into()
    }
}

// ── Switch ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Switch {
    /// Child objects held by the switch.
    pub children: Vec<Object>,
    /// Currently selected child index (defaults to 0).
    pub active: usize,
}

impl Switch {
    /// Change which child is active. This is a minimal implementation used by
    /// examples; out-of-range indices are clamped to `children.len().saturating_sub(1)`.
    pub fn switch_to(&mut self, idx: usize) {
        if self.children.is_empty() {
            self.active = 0;
        } else if idx >= self.children.len() {
            self.active = self.children.len().saturating_sub(1);
        } else {
            self.active = idx;
        }
    }

    /// Return the currently active index.
    pub fn active_index(&self) -> usize {
        self.active
    }
}

impl Default for Switch {
    fn default() -> Self {
        Self {
            children: vec![],
            active: 0,
        }
    }
}

impl Switch {
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        if let Some(child) = self.children.get(self.active.min(self.children.len().saturating_sub(1))) {
            column = column.with_child(child.into_masonry_widget());
        }
        NewWidget::new(column)
    }
}

impl From<Switch> for Element {
    fn from(s: Switch) -> Self {
        Element::Switch(s)
    }
}

impl IntoObject for Switch {
    fn into_object(self) -> Object {
        Element::from(self).into()
    }
}

// ── IntervalTimer ────────────────────────────────────────────────────────────

// Minimal interval timer used by the timer example. Generic parameter is the
// event type that will be emitted (not used by this stub).
#[derive(Debug, Clone)]
pub struct IntervalTimer<E> {
    pub interval: std::time::Duration,
    _marker: std::marker::PhantomData<E>,
}

impl<E> IntervalTimer<E> {
    pub fn from_interval(interval: std::time::Duration) -> Self {
        Self {
            interval,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E> Default for IntervalTimer<E> {
    fn default() -> Self {
        Self {
            interval: std::time::Duration::from_secs(0),
            _marker: std::marker::PhantomData,
        }
    }
}

// Provide a trivial IntoObject so elements containing an IntervalTimer can be
// converted without panicking. The timer itself doesn't correspond to any
// visual element, so we simply render an empty Text placeholder.
impl<E> IntoObject for IntervalTimer<E> {
    fn into_object(self) -> Object {
        // zero-sized representation
        Text { text: "" }.into()
    }
}
