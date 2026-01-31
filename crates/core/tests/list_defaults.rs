use snow_ui::prelude::*;

#[test]
fn list_item_appends_default() {
    let _: Vec<::snow_ui::Object> = list![Text { text: "hi" }];
}
