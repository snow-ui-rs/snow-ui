/// Minimal Server API helper used by `examples/login.rs`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServerApi {
    endpoint: String,
}

impl ServerApi {
    /// Create a new `ServerApi` pointing at the given endpoint.
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }

    /// Post JSON to the configured endpoint and return a textual response.
    ///
    /// This is a minimal synchronous stub wrapped in `async` so examples can
    /// `await` it without adding an HTTP client dependency. It simply returns
    /// a small acknowledgement that includes the posted payload.
    pub async fn post_json(&self, json: String) -> anyhow::Result<String> {
        // Minimal stub: echo the endpoint and payload. Replace with real HTTP
        // client if/when network behavior is required.
        Ok(format!(
            "{{\"endpoint\":\"{}\",\"echo\":{}}}",
            self.endpoint, json
        ))
    }
}
