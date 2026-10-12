use snow_ui::prelude::*;

fn id(object: &Object) -> WidgetId {
    match object {
        Object::Element(element) => element.id(),
    }
}

#[test]
fn every_builtin_widget_default_gets_a_distinct_id() {
    let ids = [
        Board::default().id,
        Card::default().id,
        Row::default().id,
        Text::default().id,
        TextClock::default().id,
        Button::default().id,
        Form::default().id,
        TextInput::default().id,
        Switch::default().id,
        Girl::default().id,
        IntervalTimer::<()>::default().id,
    ];
    let unique = ids.into_iter().collect::<std::collections::HashSet<_>>();

    assert_eq!(unique.len(), ids.len());

    let form = Form::default();
    assert_ne!(form.id, form.submit_button.id);
    assert_ne!(form.id, form.reset_button.id);
    assert_ne!(form.submit_button.id, form.reset_button.id);
}

#[test]
fn obj_and_list_macros_assign_ids_to_widgets_automatically() {
    let single: Object = obj!(Text { text: "from obj" });
    let tree: Object = obj!(Board {
        children: list![Text {
            text: "nested via obj"
        }],
    });
    let items: Vec<Object> = list![
        Text { text: "first" },
        Button { text: "second" },
        Row {
            children: list![Text { text: "nested" }],
        },
    ];

    assert_eq!(items.len(), 3);
    let all_ids = [
        id(&single),
        id(&tree),
        id(&items[0]),
        id(&items[1]),
        id(&items[2]),
    ];
    let unique = all_ids
        .into_iter()
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(unique.len(), all_ids.len());
    match &tree {
        Object::Element(snow_ui::Element::Board(board)) => {
            assert_eq!(board.children.len(), 1);
            assert_ne!(board.id, id(&board.children[0]));
        }
        other => panic!("expected a Board object, got {other:?}"),
    }
    match &items[2] {
        Object::Element(snow_ui::Element::Row(row)) => {
            assert_eq!(row.children.len(), 1);
            assert_ne!(row.id, id(&row.children[0]));
        }
        other => panic!("expected a Row object, got {other:?}"),
    }
}

#[test]
fn cloning_a_widget_preserves_identity() {
    let row = Row::default();
    assert_eq!(row.id, row.clone().id);
}
