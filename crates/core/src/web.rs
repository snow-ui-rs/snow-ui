use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Document, Element, Event, HtmlInputElement};

use crate::elements::Element as SnowElement;
use crate::object::{Object, World};

thread_local! {
    static RENDER_REFRESH: RefCell<Option<Box<dyn Fn()>>> = RefCell::new(None);
    static CLOCK_REFRESH: RefCell<Option<Box<dyn Fn()>>> = RefCell::new(None);
    static CLOCK_REFRESH_STARTED: RefCell<bool> = const { RefCell::new(false) };
    static RENDERED_SWITCH_INDICES: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

pub fn launch_world(world: World) {
    let world_for_refresh = world.clone();
    RENDER_REFRESH.with(|refresh| {
        *refresh.borrow_mut() = Some(Box::new(move || refresh_world(&world_for_refresh)));
    });
    configure_clock_refresh(&world);
    render_world(&world);
    let mut switch_indices = Vec::new();
    world.root.switch_active_indices(&mut switch_indices);
    RENDERED_SWITCH_INDICES.with(|rendered| *rendered.borrow_mut() = switch_indices);
}

fn configure_clock_refresh(world: &World) {
    let mut clocks = Vec::new();
    collect_web_clock_values(&world.root, &mut clocks);
    if !clocks.is_empty() {
        let world_for_clock = world.clone();
        CLOCK_REFRESH.with(|refresh| {
            *refresh.borrow_mut() = Some(Box::new(move || refresh_clocks(&world_for_clock)));
        });
        CLOCK_REFRESH_STARTED.with(|started| {
            if !*started.borrow() {
                *started.borrow_mut() = true;
                crate::runtime::interval(std::time::Duration::from_secs(1), || {
                    request_clock_refresh();
                });
            }
        });
    } else {
        CLOCK_REFRESH.with(|refresh| *refresh.borrow_mut() = None);
    }
}

pub(crate) fn request_render_refresh() {
    RENDER_REFRESH.with(|refresh| {
        if let Some(render) = refresh.borrow().as_ref() {
            render();
        }
    });
}

fn request_clock_refresh() {
    CLOCK_REFRESH.with(|refresh| {
        if let Some(update) = refresh.borrow().as_ref() {
            update();
        }
    });
}

fn refresh_clocks(world: &World) {
    let window = web_sys::window().expect("Snow UI requires a browser window");
    let document = window
        .document()
        .expect("Snow UI requires a browser document");
    let Some(root) = document.get_element_by_id("snow-root") else {
        return;
    };
    let mut values = Vec::new();
    collect_web_clock_values(&world.root, &mut values);
    let clocks = root
        .query_selector_all("[data-snow-clock]")
        .expect("failed to find Snow UI clocks");
    for (index, value) in values.iter().enumerate() {
        if let Some(clock) = clocks
            .item(index as u32)
            .and_then(|node| node.dyn_into::<Element>().ok())
        {
            clock.set_text_content(Some(value));
        }
    }
}

fn refresh_world(world: &World) {
    let mut switch_indices = Vec::new();
    world.root.switch_active_indices(&mut switch_indices);
    let structure_changed = RENDERED_SWITCH_INDICES.with(|rendered| {
        let mut rendered = rendered.borrow_mut();
        if *rendered == switch_indices {
            false
        } else {
            *rendered = switch_indices;
            true
        }
    });
    if structure_changed {
        configure_clock_refresh(world);
        render_world(world);
    } else {
        refresh_text_nodes(world);
    }
}

fn refresh_text_nodes(world: &World) {
    let window = web_sys::window().expect("Snow UI requires a browser window");
    let document = window
        .document()
        .expect("Snow UI requires a browser document");
    let Some(root) = document.get_element_by_id("snow-root") else {
        return;
    };
    let mut values = Vec::new();
    collect_web_text_values(&world.root, &mut values);
    let text_nodes = root
        .query_selector_all("[data-snow-text]")
        .expect("failed to find Snow UI text nodes");
    for (index, value) in values.iter().enumerate() {
        if let Some(node) = text_nodes
            .item(index as u32)
            .and_then(|node| node.dyn_into::<Element>().ok())
        {
            node.set_text_content(Some(value));
        }
    }
}

fn collect_web_text_values(object: &Object, values: &mut Vec<String>) {
    let Object::Element(element) = object;
    match element {
        SnowElement::Text(text) => values.push(text.visible_text()),
        SnowElement::Board(board) => {
            for child in &board.children {
                collect_web_text_values(child, values);
            }
        }
        SnowElement::Card(card) => {
            for child in &card.children {
                collect_web_text_values(child, values);
            }
        }
        SnowElement::Row(row) => {
            for child in &row.children {
                collect_web_text_values(child, values);
            }
        }
        SnowElement::Form(form) => {
            for child in &form.children {
                collect_web_text_values(child, values);
            }
        }
        SnowElement::Switch(switch_) => {
            if let Some(child) = switch_.children.get(
                switch_
                    .active_index()
                    .min(switch_.children.len().saturating_sub(1)),
            ) {
                collect_web_text_values(child, values);
            }
        }
        SnowElement::TextClock(_)
        | SnowElement::Button(_)
        | SnowElement::TextInput(_)
        | SnowElement::Girl(_) => {}
    }
}

fn collect_web_clock_values(object: &Object, values: &mut Vec<String>) {
    let Object::Element(element) = object;
    match element {
        SnowElement::TextClock(clock) => values.push(clock.visible_text()),
        SnowElement::Board(board) => {
            for child in &board.children {
                collect_web_clock_values(child, values);
            }
        }
        SnowElement::Card(card) => {
            for child in &card.children {
                collect_web_clock_values(child, values);
            }
        }
        SnowElement::Row(row) => {
            for child in &row.children {
                collect_web_clock_values(child, values);
            }
        }
        SnowElement::Form(form) => {
            for child in &form.children {
                collect_web_clock_values(child, values);
            }
        }
        SnowElement::Switch(switch_) => {
            if let Some(child) = switch_.children.get(
                switch_
                    .active_index()
                    .min(switch_.children.len().saturating_sub(1)),
            ) {
                collect_web_clock_values(child, values);
            }
        }
        SnowElement::Text(_)
        | SnowElement::Button(_)
        | SnowElement::TextInput(_)
        | SnowElement::Girl(_) => {}
    }
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
            let node = render_group(document, "div", &board.children, false, button_index);
            set_widget_id(&node, board.id);
            node
        }
        SnowElement::Card(card) => {
            let node = render_group(document, "div", &card.children, false, button_index);
            set_widget_id(&node, card.id);
            node
        }
        SnowElement::Row(row) => {
            let node = render_group(document, "div", &row.children, true, button_index);
            set_widget_id(&node, row.id);
            node
        }
        SnowElement::Text(text) => {
            let element = element_with_text(document, "span", &text.visible_text());
            element
                .set_attribute("data-snow-text", "")
                .expect("failed to tag Snow UI text");
            set_widget_id(&element, text.id);
            element
        }
        SnowElement::TextClock(clock) => {
            let element = element_with_text(document, "span", &clock.visible_text());
            element
                .set_attribute("data-snow-clock", "")
                .expect("failed to tag Snow UI clock");
            set_widget_id(&element, clock.id);
            element
        }
        SnowElement::Button(button) => {
            let button_element = element_with_text(document, "button", button.text);
            set_widget_id(&button_element, button.id);
            let widget_id = button.id;
            *button_index += 1;
            let callback =
                Closure::<dyn FnMut(Event)>::new(move |_| crate::trigger_click_handler(widget_id));
            button_element
                .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
                .expect("failed to register Snow UI button handler");
            callback.forget();
            button_element
        }
        SnowElement::Form(form) => {
            let form_element = render_group(document, "form", &form.children, false, button_index);
            set_widget_id(&form_element, form.id);
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
            set_widget_id(&submit, form.submit_button.id);
            submit
                .set_attribute("type", "button")
                .expect("failed to set Snow UI submit button type");
            let submit_id = form.submit_button.id;
            let submit_callback = Closure::<dyn FnMut(Event)>::new(move |_| {
                crate::trigger_click_handler(submit_id);
            });
            submit
                .add_event_listener_with_callback("click", submit_callback.as_ref().unchecked_ref())
                .expect("failed to register Snow UI submit handler");
            submit_callback.forget();
            buttons
                .append_child(&submit)
                .expect("failed to append Snow UI submit button");

            let reset = element_with_text(document, "button", form.reset_button.text);
            set_widget_id(&reset, form.reset_button.id);
            reset
                .set_attribute("type", "reset")
                .expect("failed to set Snow UI reset button type");
            reset
                .set_attribute("value", form.reset_button.text)
                .expect("failed to set Snow UI reset button value");
            let reset_id = form.reset_button.id;
            let reset_callback = Closure::<dyn FnMut(Event)>::new(move |_| {
                crate::trigger_click_handler(reset_id);
            });
            reset
                .add_event_listener_with_callback("click", reset_callback.as_ref().unchecked_ref())
                .expect("failed to register Snow UI reset handler");
            reset_callback.forget();
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
            set_widget_id(&wrapper, input.id);
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
                .set_attribute("data-snow-widget-id", &input.id.get().to_string())
                .expect("failed to set Snow UI input identity");
            input_element
                .set_attribute("type", input.r#type)
                .expect("failed to set Snow UI input type");
            if input.max_len > 0 {
                input_element
                    .set_attribute("maxlength", &input.max_len.to_string())
                    .expect("failed to set Snow UI input length");
            }
            let current_value = input.value.get();
            input_element
                .set_attribute("value", &input.initial_value)
                .expect("failed to set Snow UI initial input value");
            let html_input: HtmlInputElement = input_element
                .clone()
                .dyn_into()
                .expect("Snow UI input should be an HtmlInputElement");
            html_input.set_value(&current_value);
            let value_state = input.value.clone();
            let input_callback = Closure::<dyn FnMut(Event)>::new(move |_| {
                value_state.set(html_input.value());
            });
            input_element
                .add_event_listener_with_callback("input", input_callback.as_ref().unchecked_ref())
                .expect("failed to register Snow UI input handler");
            input_callback.forget();
            wrapper
                .append_child(&input_element)
                .expect("failed to append Snow UI input");
            wrapper
        }
        SnowElement::Switch(switch_) => {
            let wrapper = document
                .create_element("div")
                .expect("failed to create Snow UI switch");
            set_widget_id(&wrapper, switch_.id);
            if let Some(child) = switch_.children.get(
                switch_
                    .active_index()
                    .min(switch_.children.len().saturating_sub(1)),
            ) {
                wrapper
                    .append_child(&render_object(document, child, button_index))
                    .expect("failed to append Snow UI switch child");
            }
            wrapper
        }
        SnowElement::Girl(girl) => {
            let node = render_girl(document);
            set_widget_id(&node, girl.id);
            node
        }
    }
}

fn set_widget_id(element: &Element, id: crate::types::WidgetId) {
    element
        .set_attribute("data-snow-widget-id", &id.get().to_string())
        .expect("failed to set Snow UI widget identity");
}

fn element_with_text(document: &Document, tag: &str, text: &str) -> Element {
    let element = document
        .create_element(tag)
        .expect("failed to create Snow UI element");
    element.set_text_content(Some(text));
    element
}
