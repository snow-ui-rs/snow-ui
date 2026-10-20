//! Browser DOM primitives for Snow UI.

#[cfg(target_arch = "wasm32")]
mod browser {
    use wasm_bindgen::JsValue;
    use web_sys::{Document, Element, Window};

    pub fn window() -> Result<Window, JsValue> {
        web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))
    }

    pub fn document() -> Result<Document, JsValue> {
        window()?
            .document()
            .ok_or_else(|| JsValue::from_str("document is unavailable"))
    }

    pub fn create_element(document: &Document, tag: &str) -> Result<Element, JsValue> {
        document.create_element(tag)
    }

    pub fn append(parent: &Element, child: &Element) -> Result<(), JsValue> {
        parent.append_child(child).map(|_| ())
    }

    pub fn clear(element: &Element) -> Result<(), JsValue> {
        while let Some(child) = element.first_child() {
            element.remove_child(&child)?;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{append, clear, create_element, document, window};
#[cfg(target_arch = "wasm32")]
pub use web_sys::{Document, Element, Node, Window};
