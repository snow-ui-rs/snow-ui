#[cfg(not(target_arch = "wasm32"))]
use masonry::core::{NewWidget, WidgetTag};
#[cfg(not(target_arch = "wasm32"))]
use masonry::layout::Length;
#[cfg(not(target_arch = "wasm32"))]
use masonry::peniko::{ImageAlphaType, ImageData, ImageFormat};
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{
    Button as MasonryButton, Flex, Image, Label, SizedBox, TextInput as MasonryTextInput,
};

use crate::elements::{Element, Text, TextClock};
use crate::layout::{Board, Card, Row};
use crate::traits::IntoObject;
use crate::widgets::Girl;

#[cfg(not(target_arch = "wasm32"))]
fn sum_tag_counts(
    left: (usize, usize, usize),
    right: (usize, usize, usize),
) -> (usize, usize, usize) {
    (left.0 + right.0, left.1 + right.1, left.2 + right.2)
}

// ── Object enum ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum Object {
    Element(Element),
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct NativeTags {
    pub text: Vec<WidgetTag<Label>>,
    pub button: Vec<WidgetTag<Label>>,
    pub clock: Vec<WidgetTag<Label>>,
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Element(v) => f.debug_tuple("Element").field(v).finish(),
        }
    }
}

impl Object {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn native_tag_counts(&self) -> (usize, usize, usize) {
        match self {
            Object::Element(Element::Text(_)) => (1, 0, 0),
            Object::Element(Element::Button(_)) => (0, 1, 0),
            Object::Element(Element::TextClock(_)) => (0, 0, 1),
            Object::Element(Element::Board(board)) => board
                .children
                .iter()
                .map(Object::native_tag_counts)
                .fold((0, 0, 0), sum_tag_counts),
            Object::Element(Element::Card(card)) => card
                .children
                .iter()
                .map(Object::native_tag_counts)
                .fold((0, 0, 0), sum_tag_counts),
            Object::Element(Element::Row(row)) => row
                .children
                .iter()
                .map(Object::native_tag_counts)
                .fold((0, 0, 0), sum_tag_counts),
            Object::Element(Element::Form(form)) => {
                let children = form
                    .children
                    .iter()
                    .map(Object::native_tag_counts)
                    .fold((0, 0, 0), sum_tag_counts);
                (children.0, children.1 + 2, children.2)
            }
            Object::Element(Element::Switch(switch_)) => switch_
                .children
                .get(
                    switch_
                        .active_index()
                        .min(switch_.children.len().saturating_sub(1)),
                )
                .map_or((0, 0, 0), Object::native_tag_counts),
            Object::Element(Element::TextInput(input)) => {
                (usize::from(!input.label.is_empty()), 0, 0)
            }
            Object::Element(Element::Girl(_)) => (0, 0, 0),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn native_text_values(
        &self,
        text_values: &mut Vec<String>,
        button_values: &mut Vec<String>,
        clock_values: &mut Vec<String>,
    ) {
        match self {
            Object::Element(Element::Text(text)) => text_values.push(text.visible_text()),
            Object::Element(Element::Button(button)) => button_values.push(button.text.to_string()),
            Object::Element(Element::TextClock(clock)) => clock_values.push(clock.visible_text()),
            Object::Element(Element::Board(board)) => {
                for child in &board.children {
                    child.native_text_values(text_values, button_values, clock_values);
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &card.children {
                    child.native_text_values(text_values, button_values, clock_values);
                }
            }
            Object::Element(Element::Row(row)) => {
                for child in &row.children {
                    child.native_text_values(text_values, button_values, clock_values);
                }
            }
            Object::Element(Element::Form(form)) => {
                for child in &form.children {
                    child.native_text_values(text_values, button_values, clock_values);
                }
                button_values.push(form.submit_button.text.to_string());
                button_values.push(form.reset_button.text.to_string());
            }
            Object::Element(Element::Switch(switch_)) => {
                if let Some(child) = switch_.children.get(
                    switch_
                        .active_index()
                        .min(switch_.children.len().saturating_sub(1)),
                ) {
                    child.native_text_values(text_values, button_values, clock_values);
                }
            }
            Object::Element(Element::TextInput(input)) => {
                if !input.label.is_empty() {
                    text_values.push(input.label.to_string());
                }
            }
            Object::Element(Element::Girl(_)) => {}
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn into_masonry_widget_with_native_tags(
        &self,
        tags: &NativeTags,
        next_text: &mut usize,
        next_button: &mut usize,
        next_clock: &mut usize,
    ) -> NewWidget<Flex> {
        let mut column = Flex::column();
        match self {
            Object::Element(Element::Board(board)) => {
                for child in &board.children {
                    column = column.with_fixed(child.into_masonry_widget_with_native_tags(
                        tags,
                        next_text,
                        next_button,
                        next_clock,
                    ));
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &card.children {
                    column = column.with_fixed(child.into_masonry_widget_with_native_tags(
                        tags,
                        next_text,
                        next_button,
                        next_clock,
                    ));
                }
            }
            Object::Element(Element::Row(row)) => {
                let mut horizontal = Flex::row();
                for child in &row.children {
                    horizontal = horizontal.with_fixed(child.into_masonry_widget_with_native_tags(
                        tags,
                        next_text,
                        next_button,
                        next_clock,
                    ));
                }
                return NewWidget::new(horizontal);
            }
            Object::Element(Element::Form(form)) => {
                for child in &form.children {
                    column = column.with_fixed(child.into_masonry_widget_with_native_tags(
                        tags,
                        next_text,
                        next_button,
                        next_clock,
                    ));
                }
                let mut buttons = Flex::row();
                let submit_tag = tags.button[*next_button];
                *next_button += 1;
                buttons = buttons.with_fixed(NewWidget::new(MasonryButton::new(
                    NewWidget::new(Label::new(form.submit_button.text)).with_tag(submit_tag),
                )));
                let reset_tag = tags.button[*next_button];
                *next_button += 1;
                buttons = buttons.with_fixed(NewWidget::new(MasonryButton::new(
                    NewWidget::new(Label::new(form.reset_button.text)).with_tag(reset_tag),
                )));
                column = column.with_fixed(NewWidget::new(buttons));
            }
            Object::Element(Element::Switch(switch_)) => {
                if let Some(child) = switch_.children.get(
                    switch_
                        .active_index()
                        .min(switch_.children.len().saturating_sub(1)),
                ) {
                    column = column.with_fixed(child.into_masonry_widget_with_native_tags(
                        tags,
                        next_text,
                        next_button,
                        next_clock,
                    ));
                }
            }
            Object::Element(Element::Text(text)) => {
                let tag = tags.text[*next_text];
                *next_text += 1;
                column = column.with_fixed(
                    NewWidget::new(Label::new(text.visible_text().as_str())).with_tag(tag),
                );
            }
            Object::Element(Element::Button(button)) => {
                let tag = tags.button[*next_button];
                *next_button += 1;
                let label = NewWidget::new(Label::new(button.text)).with_tag(tag);
                column = column.with_fixed(NewWidget::new(MasonryButton::new(label)));
            }
            Object::Element(Element::TextClock(clock)) => {
                let tag = tags.clock[*next_clock];
                *next_clock += 1;
                column = column.with_fixed(
                    NewWidget::new(Label::new(clock.visible_text().as_str())).with_tag(tag),
                );
            }
            Object::Element(Element::Girl(_)) => return self.into_masonry_widget(),
            Object::Element(Element::TextInput(input)) => {
                let mut row = Flex::row();
                if !input.label.is_empty() {
                    let tag = tags.text[*next_text];
                    *next_text += 1;
                    row = row.with_fixed(NewWidget::new(Label::new(input.label)).with_tag(tag));
                }
                let input_width = input.name.chars().count() as f64 * 12.0 + 24.0;
                let input = MasonryTextInput::new("").with_placeholder(input.name);
                row = row.with_fixed(NewWidget::new(
                    SizedBox::new(NewWidget::new(input)).width(Length::const_px(input_width)),
                ));
                column = column.with_fixed(NewWidget::new(row));
            }
        }
        NewWidget::new(column)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        match self {
            Object::Element(Element::Board(board)) => board.into_masonry_widget(),
            Object::Element(Element::Card(card)) => card.into_masonry_widget(),
            Object::Element(Element::Row(row)) => row.into_masonry_widget(),
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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn text_clock_count(&self) -> usize {
        match self {
            Object::Element(Element::TextClock(_)) => 1,
            Object::Element(Element::Board(board)) => {
                board.children.iter().map(Object::text_clock_count).sum()
            }
            Object::Element(Element::Card(card)) => {
                card.children.iter().map(Object::text_clock_count).sum()
            }
            Object::Element(Element::Row(row)) => {
                row.children.iter().map(Object::text_clock_count).sum()
            }
            Object::Element(Element::Form(form)) => {
                form.children.iter().map(Object::text_clock_count).sum()
            }
            Object::Element(Element::Switch(switch_)) => switch_
                .children
                .get(
                    switch_
                        .active_index()
                        .min(switch_.children.len().saturating_sub(1)),
                )
                .map_or(0, Object::text_clock_count),
            Object::Element(
                Element::Text(_) | Element::Button(_) | Element::TextInput(_) | Element::Girl(_),
            ) => 0,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn text_clock_values(&self, values: &mut Vec<String>) {
        match self {
            Object::Element(Element::TextClock(clock)) => values.push(clock.visible_text()),
            Object::Element(Element::Board(board)) => {
                for child in &board.children {
                    child.text_clock_values(values);
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &card.children {
                    child.text_clock_values(values);
                }
            }
            Object::Element(Element::Row(row)) => {
                for child in &row.children {
                    child.text_clock_values(values);
                }
            }
            Object::Element(Element::Form(form)) => {
                for child in &form.children {
                    child.text_clock_values(values);
                }
            }
            Object::Element(Element::Switch(switch_)) => {
                if let Some(child) = switch_.children.get(
                    switch_
                        .active_index()
                        .min(switch_.children.len().saturating_sub(1)),
                ) {
                    child.text_clock_values(values);
                }
            }
            Object::Element(
                Element::Text(_) | Element::Button(_) | Element::TextInput(_) | Element::Girl(_),
            ) => {}
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget_with_clock_tags(
        &self,
        tags: &[WidgetTag<Label>],
        next_tag: &mut usize,
    ) -> NewWidget<Flex> {
        let mut column = Flex::column();
        match self {
            Object::Element(Element::Board(board)) => {
                for child in &board.children {
                    column = column
                        .with_fixed(child.into_masonry_widget_with_clock_tags(tags, next_tag));
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &card.children {
                    column = column
                        .with_fixed(child.into_masonry_widget_with_clock_tags(tags, next_tag));
                }
            }
            Object::Element(Element::Row(row)) => {
                let mut horizontal = Flex::row();
                for child in &row.children {
                    horizontal = horizontal
                        .with_fixed(child.into_masonry_widget_with_clock_tags(tags, next_tag));
                }
                return NewWidget::new(horizontal);
            }
            Object::Element(Element::Form(form)) => {
                for child in &form.children {
                    column = column
                        .with_fixed(child.into_masonry_widget_with_clock_tags(tags, next_tag));
                }
            }
            Object::Element(Element::Switch(switch_)) => {
                if let Some(child) = switch_.children.get(
                    switch_
                        .active_index()
                        .min(switch_.children.len().saturating_sub(1)),
                ) {
                    column = column
                        .with_fixed(child.into_masonry_widget_with_clock_tags(tags, next_tag));
                }
            }
            Object::Element(Element::TextClock(clock)) => {
                let tag = tags[*next_tag];
                *next_tag += 1;
                let label = NewWidget::new(Label::new(clock.visible_text().as_str())).with_tag(tag);
                column = column.with_fixed(label);
            }
            Object::Element(Element::Girl(_)) => {
                return self.into_masonry_widget();
            }
            Object::Element(Element::Text(text)) => return text.into_masonry_widget(),
            Object::Element(Element::Button(button)) => return button.into_masonry_widget(),
            Object::Element(Element::TextInput(input)) => return input.into_masonry_widget(),
        }
        NewWidget::new(column)
    }

    pub fn update_text_recursive(&mut self, text: impl Into<String>) {
        let next = text.into();
        match self {
            Object::Element(Element::Board(board)) => {
                for child in &mut board.children {
                    child.update_text_recursive(next.clone());
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &mut card.children {
                    child.update_text_recursive(next.clone());
                }
            }
            Object::Element(Element::Row(row)) => {
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
            Object::Element(Element::Board(board)) => {
                for child in &mut board.children {
                    child.update_button_text_recursive(next.clone());
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &mut card.children {
                    child.update_button_text_recursive(next.clone());
                }
            }
            Object::Element(Element::Row(row)) => {
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
            Object::Element(Element::Board(board)) => {
                for child in &board.children {
                    child.collect_button_ids(next_id, ids);
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &card.children {
                    child.collect_button_ids(next_id, ids);
                }
            }
            Object::Element(Element::Row(row)) => {
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

    pub fn has_text_clock(&self) -> bool {
        match self {
            Object::Element(Element::TextClock(_)) => true,
            Object::Element(Element::Board(board)) => {
                board.children.iter().any(Object::has_text_clock)
            }
            Object::Element(Element::Card(card)) => {
                card.children.iter().any(Object::has_text_clock)
            }
            Object::Element(Element::Row(row)) => row.children.iter().any(Object::has_text_clock),
            Object::Element(Element::Form(form)) => {
                form.children.iter().any(Object::has_text_clock)
            }
            Object::Element(Element::Switch(switch_)) => {
                switch_.children.iter().any(Object::has_text_clock)
            }
            Object::Element(
                Element::Text(_) | Element::Button(_) | Element::TextInput(_) | Element::Girl(_),
            ) => false,
        }
    }

    pub fn switch_active_indices(&self, indices: &mut Vec<usize>) {
        match self {
            Object::Element(Element::Switch(switch_)) => {
                indices.push(switch_.active_index());
                for child in &switch_.children {
                    child.switch_active_indices(indices);
                }
            }
            Object::Element(Element::Board(board)) => {
                for child in &board.children {
                    child.switch_active_indices(indices);
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &card.children {
                    child.switch_active_indices(indices);
                }
            }
            Object::Element(Element::Row(row)) => {
                for child in &row.children {
                    child.switch_active_indices(indices);
                }
            }
            Object::Element(Element::Form(form)) => {
                for child in &form.children {
                    child.switch_active_indices(indices);
                }
            }
            Object::Element(
                Element::Text(_)
                | Element::TextClock(_)
                | Element::Button(_)
                | Element::TextInput(_)
                | Element::Girl(_),
            ) => {}
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
            root: Object::from(Element::from(Board::default())),
        }
    }
}

impl From<Board> for Object {
    fn from(b: Board) -> Self {
        Object::from(Element::from(b))
    }
}

impl From<Girl> for Object {
    fn from(g: Girl) -> Self {
        Object::from(Element::from(g))
    }
}

impl From<Card> for Object {
    fn from(c: Card) -> Self {
        Object::from(Element::from(c))
    }
}

impl From<Row> for Object {
    fn from(r: Row) -> Self {
        Object::from(Element::from(r))
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
