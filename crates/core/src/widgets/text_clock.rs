#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{Flex, Label};

use crate::elements::Element;

#[derive(Debug, Clone, Default)]
pub struct TextClock {
    pub format: &'static str,
}

impl TextClock {
    pub fn visible_text(&self) -> String {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use chrono::Local;
            Local::now().format(self.format).to_string()
        }

        #[cfg(target_arch = "wasm32")]
        {
            format_web_date(self.format)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        let visible_text = self.visible_text();
        column = column.with_fixed(NewWidget::new(Label::new(visible_text.as_str())));
        NewWidget::new(column)
    }
}

#[cfg(target_arch = "wasm32")]
fn format_web_date(format: &str) -> String {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = now.get_month() + 1;
    let day = now.get_date();
    let hour = now.get_hours();
    let minute = now.get_minutes();
    let second = now.get_seconds();
    let weekday = now.get_day();
    let milliseconds = now.get_milliseconds();
    let day_of_year = day_of_year(year, month, day);
    let mut output = String::with_capacity(format.len());
    let mut chars = format.chars();

    while let Some(character) = chars.next() {
        if character != '%' {
            output.push(character);
            continue;
        }

        let Some(specifier) = chars.next() else {
            output.push('%');
            break;
        };
        match specifier {
            '%' => output.push('%'),
            'Y' => output.push_str(&format!("{year:04}")),
            'y' => output.push_str(&format!("{:02}", year.rem_euclid(100))),
            'F' => output.push_str(&format!("{year:04}-{month:02}-{day:02}")),
            'm' => output.push_str(&format!("{month:02}")),
            'b' | 'h' => output.push_str(MONTH_NAMES[month as usize - 1].0),
            'B' => output.push_str(MONTH_NAMES[month as usize - 1].1),
            'd' => output.push_str(&format!("{day:02}")),
            'e' => output.push_str(&format!("{day:2}")),
            'H' => output.push_str(&format!("{hour:02}")),
            'I' => output.push_str(&format!("{:02}", (hour % 12).max(1))),
            'R' => output.push_str(&format!("{hour:02}:{minute:02}")),
            'T' => output.push_str(&format!("{hour:02}:{minute:02}:{second:02}")),
            'M' => output.push_str(&format!("{minute:02}")),
            'S' => output.push_str(&format!("{second:02}")),
            'j' => output.push_str(&format!("{day_of_year:03}")),
            'f' => output.push_str(&format!("{milliseconds:03}000")),
            '3' if chars.next() == Some('f') => {
                output.push_str(&format!("{milliseconds:03}"));
            }
            '6' if chars.next() == Some('f') => {
                output.push_str(&format!("{milliseconds:03}000"));
            }
            '9' if chars.next() == Some('f') => {
                output.push_str(&format!("{milliseconds:03}000000"));
            }
            'p' => output.push_str(if hour < 12 { "AM" } else { "PM" }),
            'P' => output.push_str(if hour < 12 { "am" } else { "pm" }),
            'a' => output.push_str(WEEKDAY_NAMES[weekday as usize].0),
            'A' => output.push_str(WEEKDAY_NAMES[weekday as usize].1),
            'w' => output.push_str(&weekday.to_string()),
            'u' => output.push_str(&if weekday == 0 { 7 } else { weekday }.to_string()),
            'z' => {
                let offset_minutes = -now.get_timezone_offset() as i32;
                let sign = if offset_minutes >= 0 { '+' } else { '-' };
                let absolute_minutes = offset_minutes.abs();
                output.push_str(&format!(
                    "{sign}{:02}{:02}",
                    absolute_minutes / 60,
                    absolute_minutes % 60
                ));
            }
            'Z' => output.push_str(if now.get_timezone_offset() == 0.0 {
                "UTC"
            } else {
                "local"
            }),
            other => {
                output.push('%');
                output.push(other);
            }
        }
    }
    output
}

#[cfg(target_arch = "wasm32")]
fn day_of_year(year: u32, month: u32, day: u32) -> u32 {
    let month_lengths = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap_day = u32::from(year % 4 == 0 && (year % 100 != 0 || year % 400 == 0));
    let completed_months: u32 = month_lengths[..month as usize - 1].iter().sum();
    completed_months + day + u32::from(month > 2) * leap_day
}

#[cfg(target_arch = "wasm32")]
const MONTH_NAMES: [(&str, &str); 12] = [
    ("Jan", "January"),
    ("Feb", "February"),
    ("Mar", "March"),
    ("Apr", "April"),
    ("May", "May"),
    ("Jun", "June"),
    ("Jul", "July"),
    ("Aug", "August"),
    ("Sep", "September"),
    ("Oct", "October"),
    ("Nov", "November"),
    ("Dec", "December"),
];

#[cfg(target_arch = "wasm32")]
const WEEKDAY_NAMES: [(&str, &str); 7] = [
    ("Sun", "Sunday"),
    ("Mon", "Monday"),
    ("Tue", "Tuesday"),
    ("Wed", "Wednesday"),
    ("Thu", "Thursday"),
    ("Fri", "Friday"),
    ("Sat", "Saturday"),
];

impl From<TextClock> for Element {
    fn from(clock: TextClock) -> Self {
        Element::TextClock(clock)
    }
}
