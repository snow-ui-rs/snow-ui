use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Document, Element, Event, HtmlInputElement};

use crate::elements::Element as SnowElement;
use crate::object::{Object, World};

thread_local! {
    static RENDER_REFRESH: RefCell<Option<Box<dyn Fn()>>> = RefCell::new(None);
    static CLOCK_REFRESH_STARTED: RefCell<bool> = const { RefCell::new(false) };
}

pub fn launch_world(world: World) {
    let world_for_refresh = world.clone();
    RENDER_REFRESH.with(|refresh| {
        *refresh.borrow_mut() = Some(Box::new(move || render_world(&world_for_refresh)));
    });
    CLOCK_REFRESH_STARTED.with(|started| {
        if !*started.borrow() {
            *started.borrow_mut() = true;
            crate::runtime::interval(std::time::Duration::from_secs(1), || {
                request_render_refresh();
            });
        }
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
    let input_values = find_inputs(&root)
        .into_iter()
        .map(|input| input.value())
        .collect::<Vec<_>>();

    while let Some(child) = root.first_child() {
        root.remove_child(&child)
            .expect("failed to clear snow-root");
    }
    let mut button_index = 0;
    root.append_child(&render_object(&document, &world.root, &mut button_index))
        .expect("failed to mount Snow UI root");
    for (input, value) in find_inputs(&root).into_iter().zip(input_values) {
        input.set_value(&value);
    }
}

fn find_inputs(root: &Element) -> Vec<HtmlInputElement> {
    let inputs = root
        .query_selector_all("input")
        .expect("failed to find Snow UI inputs");
    (0..inputs.length())
        .filter_map(|index| inputs.item(index))
        .filter_map(|node| node.dyn_into::<HtmlInputElement>().ok())
        .collect()
}

fn render_object(document: &Document, object: &Object, button_index: &mut usize) -> Element {
    match object {
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
        SnowElement::Board(board) => {
            render_group(document, "div", &board.children, false, button_index)
        }
        SnowElement::Card(card) => {
            render_group(document, "div", &card.children, false, button_index)
        }
        SnowElement::Row(row) => render_group(document, "div", &row.children, true, button_index),
        SnowElement::Text(text) => element_with_text(document, "span", &text.visible_text()),
        SnowElement::TextClock(clock) => element_with_text(document, "span", &clock.visible_text()),
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
            let form_element = render_group(document, "form", &form.children, false, button_index);
            let buttons = document
                .create_element("div")
                .expect("failed to create Snow UI form buttons");
            buttons
                .set_attribute(
                    "style",
                    "display:flex;flex-direction:row;gap:0.5rem;margin-top:0.5rem;",
                )
                .expect("failed to style Snow UI form buttons");

            let submit = element_with_text(document, "button", form.submit_button.text);
            submit
                .set_attribute("type", "button")
                .expect("failed to set Snow UI submit button type");
            let submit_handler = form.submit_handler.clone();
            let submit_form = form.clone();
            let submit_callback = Closure::<dyn FnMut(Event)>::new(move |_| {
                let submit_handler = submit_handler.clone();
                let submit_form = submit_form.clone();
                crate::run_async(async move {
                    let _ = submit_handler.call_box(&submit_form).await;
                });
            });
            submit
                .add_event_listener_with_callback("click", submit_callback.as_ref().unchecked_ref())
                .expect("failed to register Snow UI submit handler");
            submit_callback.forget();
            buttons
                .append_child(&submit)
                .expect("failed to append Snow UI submit button");

            let reset = element_with_text(document, "button", form.reset_button.text);
            reset
                .set_attribute("type", "reset")
                .expect("failed to set Snow UI reset button type");
            buttons
                .append_child(&reset)
                .expect("failed to append Snow UI reset button");
            form_element
                .append_child(&buttons)
                .expect("failed to append Snow UI form buttons");
            form_element
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
            .get(
                switch_
                    .active_index()
                    .min(switch_.children.len().saturating_sub(1)),
            )
            .map(|child| render_object(document, child, button_index))
            .unwrap_or_else(|| element_with_text(document, "div", "")),
        SnowElement::Girl(_) => render_girl(document),
    }
}

fn element_with_text(document: &Document, tag: &str, text: &str) -> Element {
    let element = document
        .create_element(tag)
        .expect("failed to create Snow UI element");
    element.set_text_content(Some(text));
    element
}
