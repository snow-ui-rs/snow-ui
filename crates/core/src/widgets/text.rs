#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{Flex, Label};

use crate::elements::Element;
use crate::state::State;

#[derive(Clone)]
pub struct Text {
    pub text: &'static str,
    pub state: Option<std::sync::Arc<dyn Fn() -> String + Send + Sync + 'static>>,
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("text", &self.text)
            .field("state", &self.state.as_ref().map(|_| "<bound>"))
            .finish()
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: "",
            state: None,
        }
    }
}

impl Text {
    pub fn from_state<T>(state: &State<T>) -> Self
    where
        T: Clone + std::fmt::Display + Send + Sync + 'static,
    {
        let live_state = state.clone();
        Self {
            text: "",
            state: Some(std::sync::Arc::new(move || live_state.get().to_string())),
        }
    }

    pub fn visible_text(&self) -> String {
        self.state
            .as_ref()
            .map(|value| value())
            .unwrap_or_else(|| self.text.to_string())
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        let leaked: &'static str = Box::leak(text.into().into_boxed_str());
        self.text = leaked;
        self.state = None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        let visible_text = self.visible_text();
        column = column.with_fixed(NewWidget::new(Label::new(visible_text.as_str())));
        NewWidget::new(column)
    }
}

impl From<Text> for Element {
    fn from(text: Text) -> Self {
        Element::Text(text)
    }
}
