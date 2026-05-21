use taffy::prelude::*;
use taffy::TaffyTree;

fn main() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();

    let child_a = taffy
        .new_leaf(Style {
            size: Size {
                width: Dimension::length(100.0),
                height: Dimension::length(100.0),
            },
            ..Default::default()
        })
        .unwrap();

    let child_b = taffy
        .new_leaf(Style {
            size: Size {
                width: Dimension::length(80.0),
                height: Dimension::length(80.0),
            },
            ..Default::default()
        })
        .unwrap();

    let container = taffy
        .new_with_children(
            Style {
                size: Size {
                    width: Dimension::length(240.0),
                    height: Dimension::length(140.0),
                },
                flex_direction: FlexDirection::Row,
                justify_content: Some(JustifyContent::Center),
                align_items: Some(AlignItems::Center),
                ..Default::default()
            },
            &[child_a, child_b],
        )
        .unwrap();

    taffy
        .compute_layout(
            container,
            Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MaxContent,
            },
        )
        .unwrap();

    println!("Root layout: {:?}", taffy.layout(container).unwrap());
    println!("Child A layout: {:?}", taffy.layout(child_a).unwrap());
    println!("Child B layout: {:?}", taffy.layout(child_b).unwrap());
}
