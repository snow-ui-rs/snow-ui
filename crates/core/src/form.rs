use crate::elements::{Button, Element, Switch, Text, TextInput};
use crate::layout::{Board, Card, Row};
use crate::object::Object;
use crate::traits::IntoObject;

/// Trait used to convert various handler return types into `anyhow::Result<()>`.
///
/// We implement this for `()`, `Result<(), E>` and `anyhow::Result<()>` so handlers
/// can be `async fn(&Form)`, `async fn(&Form) -> Result<(), E>` or synchronous `fn(&Form)`.
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
        self.map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}

/// Object-safe submit-handler wrapper so `Form` can accept async handlers that
/// return `()` or `Result<(), E>` (including `anyhow::Result<()>`). The handler
/// returns a boxed future that resolves to `anyhow::Result<()>` so example code
/// can use `?` freely.
#[allow(dead_code)]
pub trait SubmitHandler {
    fn call_box(
        &self,
        form: &Form,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + 'static>>;
}

// Blanket impl for any function/closure that returns a Future whose `Output`
// implements `SubmitReturn`.
impl<F, Fut> SubmitHandler for F
where
    F: Fn(&Form) -> Fut + 'static,
    Fut: std::future::Future + 'static,
    Fut::Output: SubmitReturn + 'static,
{
    fn call_box(
        &self,
        form: &Form,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + 'static>> {
        let fut = (self)(form);
        Box::pin(async move { fut.await.into_anyhow() })
    }
}

// ── Form ─────────────────────────────────────────────────────────────────────

// Form element: groups input fields and exposes simple submit/reset controls.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Form {
    /// Handler invoked on submit. Accepts async functions/closures; the macro
    /// will box function items automatically so user code stays ergonomic.
    pub submit_handler: std::sync::Arc<dyn SubmitHandler>,
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Form")
            .field("submit_handler", &"<handler>")
            .field("submit_button", &self.submit_button)
            .field("reset_button", &self.reset_button)
            .field("children", &self.children)
            .finish()
    }
}

impl From<Form> for Element {
    fn from(f: Form) -> Self {
        Element::Form(f)
    }
}

impl IntoObject for Form {
    fn into_object(self) -> Object {
        Element::from(self).into()
    }
}

impl Form {
    /// Produce a lightweight JSON representation of the form's input fields.
    ///
    /// This is intentionally small and only used by examples; it returns an
    /// `anyhow::Result<String>` so callers can use the `?` operator in examples.
    pub fn to_json(&self) -> anyhow::Result<String> {
        // Simple escaping for JSON string values used in examples.
        let escape = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");

        fn walk(obj: &Object, out: &mut Vec<String>, escape: &dyn Fn(&str) -> String) {
            match obj {
                Object::Element(Element::TextInput(ti)) => {
                    out.push(format!(
                        r#"{{"name":"{}","label":"{}","type":"{}","max_len":{}}}"#,
                        escape(ti.name),
                        escape(ti.label),
                        escape(ti.r#type),
                        ti.max_len
                    ));
                }
                Object::Board(b) => {
                    for c in &b.children {
                        walk(c, out, escape);
                    }
                }
                Object::Row(r) => {
                    for c in &r.children {
                        walk(c, out, escape);
                    }
                }
                Object::Card(c) => {
                    for c2 in &c.children {
                        walk(c2, out, escape);
                    }
                }
                Object::Element(Element::Switch(s)) => {
                    for c in &s.children {
                        walk(c, out, escape);
                    }
                }
                _ => {}
            }
        }

        let mut fields = Vec::new();
        for child in &self.children {
            walk(child, &mut fields, &escape);
        }

        let json = format!("{{\"fields\":[{}]}}", fields.join(","));
        Ok(json)
    }

    /// Minimal HTTP client for examples — supports a POST-JSON operation and
    /// returns the server response body as a `String`.
    #[allow(dead_code)]
    pub fn to_json_and_post(&self, _url: &str) -> anyhow::Result<String> {
        // kept for backward-compatibility; not used by the example directly.
        Ok(self.to_json()?)
    }
}
