#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::layout::Dim;
#[cfg(not(target_arch = "wasm32"))]
use masonry::properties::Dimensions;
#[cfg(not(target_arch = "wasm32"))]
use masonry::properties::types::MainAxisAlignment;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::{Button as MasonryButton, Flex};

use crate::elements::{Button, Element};
use crate::object::Object;
use crate::traits::IntoObject;
use crate::types::{HAlign, WidgetId};

trait SubmitReturn {
    fn into_anyhow(self) -> anyhow::Result<()>;
}

impl SubmitReturn for () {
    fn into_anyhow(self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<E> SubmitReturn for Result<(), E>
where
    E: std::fmt::Debug + Send + Sync + 'static,
{
    fn into_anyhow(self) -> anyhow::Result<()> {
        self.map_err(|error| anyhow::anyhow!("{:?}", error))
    }
}

pub trait SubmitHandler: Send + Sync {
    fn call_box(
        &self,
        form: &Form,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'static>>;
}

impl<F, Fut> SubmitHandler for F
where
    F: Fn(&Form) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future + Send + 'static,
    Fut::Output: SubmitReturn + 'static,
{
    fn call_box(
        &self,
        form: &Form,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'static>>
    {
        let future = (self)(form);
        Box::pin(async move { future.await.into_anyhow() })
    }
}

#[derive(Clone)]
pub struct Form {
    pub id: WidgetId,
    pub submit_handler: std::sync::Arc<dyn SubmitHandler + Send + Sync>,
    pub submit_button: Button,
    pub reset_button: Button,
    pub h_align: HAlign,
    pub children: Vec<Object>,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            id: WidgetId::new(),
            submit_handler: std::sync::Arc::new(|_form: &Form| Box::pin(async move {})),
            submit_button: Button::default(),
            reset_button: Button::default(),
            h_align: HAlign::Left,
            children: vec![],
        }
    }
}

impl std::fmt::Debug for Form {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Form")
            .field("submit_handler", &"<handler>")
            .field("id", &self.id)
            .field("submit_button", &self.submit_button)
            .field("reset_button", &self.reset_button)
            .field("h_align", &self.h_align)
            .field("children", &self.children)
            .finish()
    }
}

impl Form {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        for child in &self.children {
            column = crate::object::add_masonry_child(column, child, child.into_masonry_widget());
        }
        let mut buttons = Flex::row().main_axis_alignment(MainAxisAlignment::Center);
        buttons = buttons.with_fixed(NewWidget::new(MasonryButton::with_text(
            self.submit_button.text,
        )));
        buttons = buttons.with_fixed(NewWidget::new(MasonryButton::with_text(
            self.reset_button.text,
        )));
        column =
            column.with_fixed(NewWidget::new(buttons).with_props(Dimensions::width(Dim::Stretch)));
        NewWidget::new(column).with_props(Dimensions::width(Dim::MaxContent))
    }

    fn collect_fields<'a>(obj: &'a Object, output: &mut Vec<&'a crate::widgets::TextInput>) {
        match obj {
            Object::Element(Element::TextInput(input)) => {
                output.push(input);
            }
            Object::Element(Element::Board(board)) => {
                for child in &board.children {
                    Self::collect_fields(child, output);
                }
            }
            Object::Element(Element::Row(row)) => {
                for child in &row.children {
                    Self::collect_fields(child, output);
                }
            }
            Object::Element(Element::Card(card)) => {
                for child in &card.children {
                    Self::collect_fields(child, output);
                }
            }
            Object::Element(Element::Switch(switch_)) => {
                if let Some(child) = switch_.children.get(
                    switch_
                        .active_index()
                        .min(switch_.children.len().saturating_sub(1)),
                ) {
                    Self::collect_fields(child, output);
                }
            }
            _ => {}
        }
    }

    /// Return field names and their current user-entered values, preserving field order.
    pub fn values(&self) -> Vec<(String, String)> {
        let mut fields = Vec::new();
        for child in &self.children {
            Self::collect_fields(child, &mut fields);
        }
        fields
            .into_iter()
            .map(|field| (field.name.to_owned(), field.value.get()))
            .collect()
    }

    /// Serialize form metadata and the current values as valid JSON.
    pub fn to_json(&self) -> anyhow::Result<String> {
        let mut fields = Vec::new();
        for child in &self.children {
            Self::collect_fields(child, &mut fields);
        }
        let fields = fields
            .into_iter()
            .map(|field| {
                serde_json::json!({
                    "name": field.name,
                    "label": field.label,
                    "type": field.r#type,
                    "max_len": field.max_len,
                    "value": field.value.get(),
                })
            })
            .collect::<Vec<_>>();
        Ok(serde_json::json!({ "fields": fields }).to_string())
    }

    /// Restore all fields in this form to their configured initial values.
    pub fn reset_values(&self) {
        fn reset(obj: &Object) {
            match obj {
                Object::Element(Element::TextInput(input)) => {
                    input.value.set(input.initial_value.clone());
                }
                Object::Element(Element::Board(board)) => {
                    for child in &board.children {
                        reset(child);
                    }
                }
                Object::Element(Element::Row(row)) => {
                    for child in &row.children {
                        reset(child);
                    }
                }
                Object::Element(Element::Card(card)) => {
                    for child in &card.children {
                        reset(child);
                    }
                }
                Object::Element(Element::Switch(switch_)) => {
                    if let Some(child) = switch_.children.get(
                        switch_
                            .active_index()
                            .min(switch_.children.len().saturating_sub(1)),
                    ) {
                        reset(child);
                    }
                }
                _ => {}
            }
        }

        for child in &self.children {
            reset(child);
        }
    }
}

impl From<Form> for Element {
    fn from(form: Form) -> Self {
        Element::Form(form)
    }
}

impl IntoObject for Form {
    fn into_object(self) -> Object {
        let submit_id = self.submit_button.id;
        let submit_handler = self.submit_handler.clone();
        let submit_form = self.clone();
        crate::register_click_handler(submit_id, move || {
            let submit_handler = submit_handler.clone();
            let submit_form = submit_form.clone();
            #[cfg(not(target_arch = "wasm32"))]
            let join_handle = std::thread::spawn(move || {
                crate::run_async(async move {
                    let _ = submit_handler.call_box(&submit_form).await;
                });
            });
            #[cfg(not(target_arch = "wasm32"))]
            let _ = join_handle.join();
            #[cfg(target_arch = "wasm32")]
            crate::run_async(async move {
                let _ = submit_handler.call_box(&submit_form).await;
            });
        });
        let reset_id = self.reset_button.id;
        let reset_form = self.clone();
        crate::register_click_handler(reset_id, move || reset_form.reset_values());
        Object::from(Element::from(self))
    }
}
