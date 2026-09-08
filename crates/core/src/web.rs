use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Document, Element, Event};

use crate::elements::Element as SnowElement;
use crate::object::{Object, World};

thread_local! {
    static RENDER_REFRESH: RefCell<Option<Box<dyn Fn()>>> = RefCell::new(None);
}

pub fn launch_world(world: World) {
    let world_for_refresh = world.clone();
    RENDER_REFRESH.with(|refresh| {
        *refresh.borrow_mut() = Some(Box::new(move || render_world(&world_for_refresh)));
    });
    render_world(&world);
}

pub(crate) fn request_render_refresh() {
    RENDER_REFRESH.with(|refresh| {
        if let Some(render) = refresh.borrow().as_ref() {
            render();
        }
    });
}

fn render_world(world: &World) {
    let window = web_sys::window().expect("Snow UI requires a browser window");
    let document = window
        .document()
        .expect("Snow UI requires a browser document");
    let root = document
        .get_element_by_id("snow-root")
        .expect("Snow UI requires an element with id=\"snow-root\"");

    while let Some(child) = root.first_child() {
        root.remove_child(&child)
            .expect("failed to clear snow-root");
    }
    let mut button_index = 0;
    root.append_child(&render_object(&document, &world.root, &mut button_index))
        .expect("failed to mount Snow UI root");
}

fn render_object(document: &Document, object: &Object, button_index: &mut usize) -> Element {
    match object {
        Object::Board(board) => render_group(document, "div", &board.children, false, button_index),
        Object::Card(card) => render_group(document, "div", &card.children, false, button_index),
        Object::Row(row) => render_group(document, "div", &row.children, true, button_index),
        Object::Girl(_) => render_girl(document),
        Object::DynamicText { value } => element_with_text(document, "span", &value()),
        Object::Element(element) => render_element(document, element, button_index),
    }
}

fn render_girl(document: &Document) -> Element {
    let image = document
        .create_element("img")
        .expect("failed to create Snow UI girl image");
    image
        .set_attribute("src", "/assets/girl.png")
        .expect("failed to set Snow UI girl image source");
    image
        .set_attribute("alt", "Girl")
        .expect("failed to set Snow UI girl image alt text");
    image
        .set_attribute(
            "style",
            "display:block;max-width:100%;height:auto;object-fit:contain;",
        )
        .expect("failed to style Snow UI girl image");
    image
}

fn render_group(
    document: &Document,
    tag: &str,
    children: &[Object],
    horizontal: bool,
    button_index: &mut usize,
) -> Element {
    let element = document
        .create_element(tag)
        .expect("failed to create Snow UI container");
    let style = if horizontal {
        "display:flex;flex-direction:row;gap:0.5rem;"
    } else {
        "display:flex;flex-direction:column;gap:0.5rem;"
    };
    element
        .set_attribute("style", style)
        .expect("failed to style Snow UI container");
    for child in children {
        element
            .append_child(&render_object(document, child, button_index))
            .expect("failed to append Snow UI child");
    }
    element
}

fn render_element(document: &Document, element: &SnowElement, button_index: &mut usize) -> Element {
    match element {
        SnowElement::Text(text) => element_with_text(document, "span", &text.visible_text()),
        SnowElement::TextClock(clock) => element_with_text(document, "span", clock.format),
        SnowElement::Button(button) => {
            let button_element = element_with_text(document, "button", button.text);
            let current_index = *button_index;
            *button_index += 1;
            let callback = Closure::<dyn FnMut(Event)>::new(move |_| {
                crate::trigger_click_handler(current_index)
            });
            button_element
                .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
                .expect("failed to register Snow UI button handler");
            callback.forget();
            button_element
        }
        SnowElement::Form(form) => {
            render_group(document, "form", &form.children, false, button_index)
        }
        SnowElement::TextInput(input) => {
            let wrapper = document
                .create_element("label")
                .expect("failed to create Snow UI input label");
            if !input.label.is_empty() {
                wrapper.set_text_content(Some(input.label));
            }
            let input_element = document
                .create_element("input")
                .expect("failed to create Snow UI input");
            input_element
                .set_attribute("name", input.name)
                .expect("failed to set Snow UI input name");
            input_element
                .set_attribute("type", input.r#type)
                .expect("failed to set Snow UI input type");
            if input.max_len > 0 {
                input_element
                    .set_attribute("maxlength", &input.max_len.to_string())
                    .expect("failed to set Snow UI input length");
            }
            wrapper
                .append_child(&input_element)
                .expect("failed to append Snow UI input");
            wrapper
        }
        SnowElement::Switch(switch_) => switch_
            .children
            .get(switch_.active.min(switch_.children.len().saturating_sub(1)))
            .map(|child| render_object(document, child, button_index))
            .unwrap_or_else(|| element_with_text(document, "div", "")),
    }
}

fn element_with_text(document: &Document, tag: &str, text: &str) -> Element {
    let element = document
        .create_element(tag)
        .expect("failed to create Snow UI element");
    element.set_text_content(Some(text));
    element
}
