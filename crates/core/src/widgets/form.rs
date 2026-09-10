#[cfg(not(target_arch = "wasm32"))]
use masonry::core::NewWidget;
#[cfg(not(target_arch = "wasm32"))]
use masonry::widgets::Flex;

use crate::elements::{Button, Element};
use crate::object::Object;
use crate::traits::IntoObject;

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
    pub submit_handler: std::sync::Arc<dyn SubmitHandler + Send + Sync>,
    pub submit_button: Button,
    pub reset_button: Button,
    pub children: Vec<Object>,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            submit_handler: std::sync::Arc::new(|_form: &Form| Box::pin(async move {})),
            submit_button: Button::default(),
            reset_button: Button::default(),
            children: vec![],
        }
    }
}

impl std::fmt::Debug for Form {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Form")
            .field("submit_handler", &"<handler>")
            .field("submit_button", &self.submit_button)
            .field("reset_button", &self.reset_button)
            .field("children", &self.children)
            .finish()
    }
}

impl Form {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn into_masonry_widget(&self) -> NewWidget<Flex> {
        let mut column = Flex::column();
        for child in &self.children {
            column = column.with_fixed(child.into_masonry_widget());
        }
        NewWidget::new(column)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        let escape = |value: &str| value.replace('\\', "\\\\").replace('"', "\\\"");

        fn walk(obj: &Object, output: &mut Vec<String>, escape: &dyn Fn(&str) -> String) {
            match obj {
                Object::Element(Element::TextInput(input)) => {
                    output.push(format!(
                        r#"{{"name":"{}","label":"{}","type":"{}","max_len":{}}}"#,
                        escape(input.name),
                        escape(input.label),
                        escape(input.r#type),
                        input.max_len
                    ));
                }
                Object::Board(board) => {
                    for child in &board.children {
                        walk(child, output, escape);
                    }
                }
                Object::Row(row) => {
                    for child in &row.children {
                        walk(child, output, escape);
                    }
                }
                Object::Card(card) => {
                    for child in &card.children {
                        walk(child, output, escape);
                    }
                }
                Object::Element(Element::Switch(switch_)) => {
                    for child in &switch_.children {
                        walk(child, output, escape);
                    }
                }
                _ => {}
            }
        }

        let mut fields = Vec::new();
        for child in &self.children {
            walk(child, &mut fields, &escape);
        }
        Ok(format!(r#"{{"fields":[{}]}}"#, fields.join(",")))
    }
}

impl From<Form> for Element {
    fn from(form: Form) -> Self {
        Element::Form(form)
    }
}

impl IntoObject for Form {
    fn into_object(self) -> Object {
        Object::from(Element::from(self))
    }
}
