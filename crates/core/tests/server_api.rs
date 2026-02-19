use snow_ui::prelude::*;

#[test]
fn server_api_post_echoes_payload() {
    let api = ServerApi::new("https://example.local/post");
    let resp = futures::executor::block_on(api.post_json("{\"a\":1}".to_string())).unwrap();
    assert!(resp.contains("example.local"));
    assert!(resp.contains("{\"a\":1}"));
}
