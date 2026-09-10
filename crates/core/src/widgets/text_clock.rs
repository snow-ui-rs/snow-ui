#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{Flex, Label};

use crate::elements::Element;

#[derive(Debug, Clone)]
pub struct TextClock {
    pub format: &'static str,
}

impl Default for TextClock {
    fn default() -> Self {
        Self { format: "" }
    }
}

impl TextClock {
    pub fn visible_text(&self) -> String {
        let (hours, minutes, seconds) = local_clock_parts();
        let mut output = String::with_capacity(self.format.len());
        let mut chars = self.format.chars();
        while let Some(ch) = chars.next() {
            if ch != '%' {
                output.push(ch);
                continue;
            }
            match chars.next() {
                Some('H') => output.push_str(&format!("{hours:02}")),
                Some('M') => output.push_str(&format!("{minutes:02}")),
                Some('S') => output.push_str(&format!("{seconds:02}")),
                Some('%') => output.push('%'),
                Some(other) => {
                    output.push('%');
                    output.push(other);
                }
                None => output.push('%'),
            }
        }
        output
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        let visible_text = self.visible_text();
        column = column.with_fixed(NewWidget::new(Label::new(visible_text.as_str())));
        NewWidget::new(column)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn local_clock_parts() -> (u32, u32, u32) {
    use chrono::{Local, Timelike};
    let now = Local::now();
    (now.hour(), now.minute(), now.second())
}

#[cfg(target_arch = "wasm32")]
fn local_clock_parts() -> (u32, u32, u32) {
    let now = js_sys::Date::new_0();
    (now.get_hours(), now.get_minutes(), now.get_seconds())
}

impl From<TextClock> for Element {
    fn from(clock: TextClock) -> Self {
        Element::TextClock(clock)
    }
}
