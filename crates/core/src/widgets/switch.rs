#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::Flex;

use crate::elements::Element;
use crate::object::Object;

#[derive(Debug, Clone, Default)]
pub struct Switch {
    pub children: Vec<Object>,
    pub active: usize,
}

fn shared_active_states() -> &'static std::sync::Mutex<std::collections::HashMap<u64, usize>> {
    static STATES: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<u64, usize>>> =
        std::sync::OnceLock::new();
    STATES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn state_key(children: &[Object]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{children:?}").hash(&mut hasher);
    hasher.finish()
}

impl Switch {
    pub fn switch_to(&mut self, idx: usize) {
        let active = if self.children.is_empty() {
            0
        } else {
            idx.min(self.children.len() - 1)
        };
        self.active = active;
        shared_active_states()
            .lock()
            .unwrap()
            .insert(state_key(&self.children), active);
    }

    pub fn active_index(&self) -> usize {
        shared_active_states()
            .lock()
            .unwrap()
            .get(&state_key(&self.children))
            .copied()
            .unwrap_or(self.active)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        if let Some(child) = self.children.get(
            self.active_index()
                .min(self.children.len().saturating_sub(1)),
        ) {
            column = column.with_fixed(child.into_masonry_widget());
        }
        NewWidget::new(column)
    }
}

impl From<Switch> for Element {
    fn from(switch: Switch) -> Self {
        Element::Switch(switch)
    }
}

impl crate::traits::IntoObject for Switch {
    fn into_object(self) -> Object {
        Object::from(Element::from(self))
    }
}

#[cfg(test)]
mod tests {
    use super::Switch;
    use crate::elements::Text;
    use crate::object::Object;

    #[test]
    fn cloned_switches_share_active_selection() {
        let mut switch = Switch {
            children: vec![
                Object::from(Text {
                    text: "first",
                    ..Text::default()
                }),
                Object::from(Text {
                    text: "second",
                    ..Text::default()
                }),
            ],
            active: 0,
        };
        let clone = switch.clone();

        switch.switch_to(1);

        assert_eq!(clone.active_index(), 1);
    }
}
