use std::collections::BTreeMap;

use crate::{LayoutTree, LayoutTreeError, RefreshBatch, WidgetId};

/// Axis-aligned rectangle in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    fn intersects_or_touches(self, other: Self) -> bool {
        self.x <= other.x + other.width
            && other.x <= self.x + self.width
            && self.y <= other.y + other.height
            && other.y <= self.y + self.height
    }

    fn union(self, other: Self) -> Self {
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);
        Self {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }
}

/// Linear RGBA color with components in the `0.0..=1.0` range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

/// A retained solid rectangle in the native paint scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaintNode {
    pub id: WidgetId,
    pub bounds: Rect,
    pub color: Color,
    pub z_index: i32,
}

/// Border drawn inside a rounded rectangle's bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Border {
    pub width: f32,
    pub color: Color,
}

/// Soft shadow drawn behind a rounded rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub color: Color,
}

/// Vertical face shading that gives a rounded rectangle a raised appearance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bevel {
    /// Brightening/darkening strength, clamped to `0.0..=1.0` by the renderer.
    pub intensity: f32,
}

/// A retained rounded rectangle with an optional inside border.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoundedRectNode {
    pub id: WidgetId,
    pub bounds: Rect,
    pub corner_radius: f32,
    pub color: Color,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub bevel: Option<Bevel>,
    pub z_index: i32,
}

impl RoundedRectNode {
    pub(crate) fn visual_bounds(self) -> Rect {
        let Some(shadow) = self.shadow else {
            return self.bounds;
        };
        let offset_x = if shadow.offset_x.is_finite() {
            shadow.offset_x
        } else {
            0.0
        };
        let offset_y = if shadow.offset_y.is_finite() {
            shadow.offset_y
        } else {
            0.0
        };
        let blur_radius = if shadow.blur_radius.is_finite() {
            shadow.blur_radius.max(0.0)
        } else {
            0.0
        };
        let outset = blur_radius * 3.0 + 1.0;
        Rect {
            x: self.bounds.x + offset_x - outset,
            y: self.bounds.y + offset_y - outset,
            width: self.bounds.width + outset * 2.0,
            height: self.bounds.height + outset * 2.0,
        }
    }
}

/// A retained text run in the native paint scene.
#[derive(Debug, Clone, PartialEq)]
pub struct TextNode {
    pub id: WidgetId,
    pub text: String,
    pub bounds: Rect,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Color,
    pub z_index: i32,
}

/// A renderer-neutral solid-rectangle draw command.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillRect {
    pub id: WidgetId,
    pub bounds: Rect,
    pub color: Color,
    pub z_index: i32,
}

/// The work required to paint one frame.
#[derive(Debug, Default, PartialEq)]
pub struct PaintFrame {
    pub full: bool,
    pub damage: Vec<Rect>,
    pub commands: Vec<FillRect>,
    pub rounded_rects: Vec<RoundedRectNode>,
    pub text: Vec<TextNode>,
}

/// Retained paint nodes with damage tracking for local redraws.
#[derive(Debug, Default)]
pub struct PaintScene {
    nodes: BTreeMap<WidgetId, PaintNode>,
    rounded_rects: BTreeMap<WidgetId, RoundedRectNode>,
    text_nodes: BTreeMap<WidgetId, TextNode>,
    pending_damage: Vec<Rect>,
}

impl PaintScene {
    /// Synchronize retained paint-node bounds with the latest computed layout.
    ///
    /// All layout queries are completed before any nodes are changed, so an
    /// unknown widget leaves the scene untouched. Changed bounds mark both the
    /// previous and new rectangles as damage for the next frame.
    pub fn sync_layout(&mut self, layout: &LayoutTree) -> Result<(), LayoutTreeError> {
        let mut updates =
            Vec::with_capacity(self.nodes.len() + self.rounded_rects.len() + self.text_nodes.len());
        for id in self
            .nodes
            .keys()
            .chain(self.rounded_rects.keys())
            .chain(self.text_nodes.keys())
        {
            let rect = layout.layout(*id)?;
            updates.push((
                *id,
                Rect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                },
            ));
        }

        for (id, bounds) in updates {
            if let Some(node) = self.nodes.get_mut(&id) {
                if node.bounds != bounds {
                    self.pending_damage.push(node.bounds);
                    self.pending_damage.push(bounds);
                    node.bounds = bounds;
                }
            }
            if let Some(node) = self.rounded_rects.get_mut(&id) {
                if node.bounds != bounds {
                    self.pending_damage.push(node.visual_bounds());
                    node.bounds = bounds;
                    self.pending_damage.push(node.visual_bounds());
                }
            }
            if let Some(node) = self.text_nodes.get_mut(&id) {
                if node.bounds != bounds {
                    self.pending_damage.push(node.bounds);
                    self.pending_damage.push(bounds);
                    node.bounds = bounds;
                }
            }
        }
        Ok(())
    }

    /// Add or update a rectangle, recording both old and new bounds as damage.
    pub fn upsert(&mut self, node: PaintNode) {
        if let Some(previous) = self.rounded_rects.remove(&node.id) {
            self.add_damage(previous.visual_bounds());
        }
        let previous = self.nodes.insert(node.id, node);
        if let Some(previous) = previous {
            if previous != node {
                self.add_damage(previous.bounds);
                self.add_damage(node.bounds);
            }
        } else {
            self.add_damage(node.bounds);
        }
    }

    /// Add or update a rounded rectangle, recording old and new bounds as damage.
    pub fn upsert_rounded_rect(&mut self, node: RoundedRectNode) {
        if let Some(previous) = self.nodes.remove(&node.id) {
            self.add_damage(previous.bounds);
        }
        let previous = self.rounded_rects.insert(node.id, node);
        if let Some(previous) = previous {
            if previous != node {
                self.add_damage(previous.visual_bounds());
                self.add_damage(node.visual_bounds());
            }
        } else {
            self.add_damage(node.visual_bounds());
        }
    }

    /// Remove a node and mark its former bounds for clearing.
    pub fn remove(&mut self, id: WidgetId) -> Option<PaintNode> {
        let removed = self.nodes.remove(&id)?;
        self.add_damage(removed.bounds);
        Some(removed)
    }

    /// Remove a rounded rectangle and mark its former bounds for clearing.
    pub fn remove_rounded_rect(&mut self, id: WidgetId) -> Option<RoundedRectNode> {
        let removed = self.rounded_rects.remove(&id)?;
        self.add_damage(removed.visual_bounds());
        Some(removed)
    }

    /// Add or update a text run, recording old and new bounds as damage.
    pub fn upsert_text(&mut self, node: TextNode) {
        let previous = self.text_nodes.insert(node.id, node.clone());
        if let Some(previous) = previous {
            if previous != node {
                self.add_damage(previous.bounds);
                self.add_damage(node.bounds);
            }
        } else {
            self.add_damage(node.bounds);
        }
    }

    /// Remove a text run and mark its former bounds for clearing.
    pub fn remove_text(&mut self, id: WidgetId) -> Option<TextNode> {
        let removed = self.text_nodes.remove(&id)?;
        self.add_damage(removed.bounds);
        Some(removed)
    }

    /// Remove any retained shape and text node for a widget, damaging all former
    /// bounds. This is useful when `LayoutTree::remove_subtree` returns the IDs
    /// that have left the tree.
    pub fn remove_widget(&mut self, id: WidgetId) -> (Option<PaintNode>, Option<TextNode>) {
        let (rect, _, text) = self.remove_widget_with_shapes(id);
        (rect, text)
    }

    /// Remove all retained shapes and text for a widget, including rounded
    /// rectangles, and return the removed nodes.
    pub fn remove_widget_with_shapes(
        &mut self,
        id: WidgetId,
    ) -> (Option<PaintNode>, Option<RoundedRectNode>, Option<TextNode>) {
        let rect = self.remove(id);
        let rounded_rect = self.remove_rounded_rect(id);
        let text = self.remove_text(id);
        (rect, rounded_rect, text)
    }

    /// Build a frame from accumulated geometry changes and paint invalidations.
    /// Shapes and text include retained nodes intersecting damage regions.
    /// Shapes are sorted together by `(z_index, WidgetId)`.
    pub fn take_frame(&mut self, viewport: Rect, refresh: &RefreshBatch) -> PaintFrame {
        let full = refresh.full;
        let mut damage = if full {
            self.pending_damage.clear();
            vec![viewport]
        } else {
            std::mem::take(&mut self.pending_damage)
        };

        if !full {
            for id in &refresh.paint_nodes {
                if let Some(node) = self.nodes.get(id) {
                    damage.push(node.bounds);
                }
                if let Some(node) = self.rounded_rects.get(id) {
                    damage.push(node.visual_bounds());
                }
                if let Some(node) = self.text_nodes.get(id) {
                    damage.push(node.bounds);
                }
            }
        }

        damage.retain(|rect| !rect.is_empty());
        let damage = merge_damage(damage);
        let mut commands: Vec<_> = self
            .nodes
            .values()
            .filter(|node| {
                damage
                    .iter()
                    .any(|region| node.bounds.intersects_or_touches(*region))
            })
            .map(|node| FillRect {
                id: node.id,
                bounds: node.bounds,
                color: node.color,
                z_index: node.z_index,
            })
            .collect();
        commands.sort_by_key(|command| (command.z_index, command.id));
        let mut rounded_rects: Vec<_> = self
            .rounded_rects
            .values()
            .filter(|node| {
                damage
                    .iter()
                    .any(|region| node.visual_bounds().intersects_or_touches(*region))
            })
            .copied()
            .collect();
        rounded_rects.sort_by_key(|command| (command.z_index, command.id));
        let mut text: Vec<_> = self
            .text_nodes
            .values()
            .filter(|node| {
                damage
                    .iter()
                    .any(|region| node.bounds.intersects_or_touches(*region))
            })
            .cloned()
            .collect();
        text.sort_by_key(|node| (node.z_index, node.id));

        PaintFrame {
            full,
            damage,
            commands,
            rounded_rects,
            text,
        }
    }

    fn add_damage(&mut self, rect: Rect) {
        if !rect.is_empty() {
            self.pending_damage.push(rect);
        }
    }
}

fn merge_damage(regions: Vec<Rect>) -> Vec<Rect> {
    let mut merged: Vec<Rect> = Vec::new();
    for mut candidate in regions {
        let mut index = 0;
        while index < merged.len() {
            if candidate.intersects_or_touches(merged[index]) {
                candidate = candidate.union(merged.swap_remove(index));
                index = 0;
            } else {
                index += 1;
            }
        }
        merged.push(candidate);
    }
    merged.sort_by(|left, right| left.y.total_cmp(&right.y).then(left.x.total_cmp(&right.x)));
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayoutNode, LayoutTree};
    use taffy::{Dimension, Display, FlexDirection, Size, Style};

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn node(id: u64, bounds: Rect, z_index: i32) -> PaintNode {
        PaintNode {
            id: WidgetId::from_raw(id),
            bounds,
            color: Color::rgba(0.2, 0.4, 0.8, 1.0),
            z_index,
        }
    }

    #[test]
    fn sync_layout_updates_bounds_and_marks_old_and_new_damage() {
        let spacer_id = WidgetId::from_raw(101);
        let target_id = WidgetId::from_raw(102);
        let root = LayoutNode::with_children(
            WidgetId::from_raw(100),
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: Dimension::length(100.0),
                    height: Dimension::length(40.0),
                },
                ..Style::default()
            },
            vec![
                LayoutNode::new(
                    spacer_id,
                    Style {
                        size: Size {
                            width: Dimension::length(12.0),
                            height: Dimension::length(20.0),
                        },
                        ..Style::default()
                    },
                ),
                LayoutNode::new(
                    target_id,
                    Style {
                        size: Size {
                            width: Dimension::length(20.0),
                            height: Dimension::length(20.0),
                        },
                        ..Style::default()
                    },
                ),
            ],
        );
        let mut layout = LayoutTree::new(root).unwrap();
        layout.compute_layout(100.0, 40.0).unwrap();

        let mut scene = PaintScene::default();
        scene.upsert(node(102, rect(0.0, 0.0, 20.0, 20.0), 0));
        let _ = scene.take_frame(rect(0.0, 0.0, 100.0, 40.0), &RefreshBatch::default());

        scene.sync_layout(&layout).unwrap();
        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 40.0), &RefreshBatch::default());
        assert_eq!(frame.commands[0].bounds, rect(12.0, 0.0, 20.0, 20.0));
        assert!(!frame.damage.is_empty());
    }

    #[test]
    fn remove_widget_clears_rectangle_and_text_nodes() {
        let id = WidgetId::from_raw(90);
        let bounds = rect(4.0, 5.0, 30.0, 12.0);
        let mut scene = PaintScene::default();
        scene.upsert(node(id.get(), bounds, 0));
        scene.upsert_text(TextNode {
            id,
            text: "label".into(),
            bounds,
            font_size: 12.0,
            line_height: 14.0,
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            z_index: 1,
        });

        let (rect_node, text_node) = scene.remove_widget(id);
        assert!(rect_node.is_some());
        assert!(text_node.is_some());
        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());
        assert_eq!(frame.commands, Vec::<FillRect>::new());
        assert!(frame.rounded_rects.is_empty());
        assert_eq!(frame.text, Vec::<TextNode>::new());
        assert_eq!(frame.damage, vec![bounds]);
    }

    #[test]
    fn rounded_rect_updates_and_removal_damage_its_bounds() {
        let id = WidgetId::from_raw(91);
        let bounds = rect(4.0, 5.0, 30.0, 20.0);
        let original = RoundedRectNode {
            id,
            bounds,
            corner_radius: 6.0,
            color: Color::rgba(0.2, 0.4, 0.8, 1.0),
            border: None,
            shadow: None,
            bevel: None,
            z_index: 1,
        };
        let mut scene = PaintScene::default();
        scene.upsert_rounded_rect(original);
        let _ = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());

        let updated = RoundedRectNode {
            corner_radius: 10.0,
            border: Some(Border {
                width: 2.0,
                color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            }),
            shadow: Some(Shadow {
                offset_x: 0.0,
                offset_y: 3.0,
                blur_radius: 4.0,
                color: Color::rgba(0.0, 0.0, 0.0, 0.3),
            }),
            bevel: Some(Bevel { intensity: 0.1 }),
            ..original
        };
        scene.upsert_rounded_rect(updated);
        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());
        assert_eq!(frame.rounded_rects, vec![updated]);
        assert_eq!(frame.damage, vec![updated.visual_bounds()]);

        assert_eq!(scene.remove_widget_with_shapes(id).1, Some(updated));
        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());
        assert!(frame.rounded_rects.is_empty());
        assert_eq!(frame.damage, vec![updated.visual_bounds()]);
    }

    #[test]
    fn rectangle_shapes_share_z_order() {
        let rect_node = node(92, rect(0.0, 0.0, 20.0, 20.0), 2);
        let rounded_rect = RoundedRectNode {
            id: WidgetId::from_raw(93),
            bounds: rect(0.0, 0.0, 20.0, 20.0),
            corner_radius: 4.0,
            color: Color::rgba(0.2, 0.4, 0.8, 1.0),
            border: None,
            shadow: None,
            bevel: None,
            z_index: 1,
        };
        let mut scene = PaintScene::default();
        scene.upsert(rect_node);
        scene.upsert_rounded_rect(rounded_rect);

        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());
        assert_eq!(frame.rounded_rects, vec![rounded_rect]);
        assert_eq!(
            frame.commands,
            vec![FillRect {
                id: rect_node.id,
                bounds: rect_node.bounds,
                color: rect_node.color,
                z_index: rect_node.z_index,
            }]
        );
        assert!(frame.rounded_rects[0].z_index < frame.commands[0].z_index);
    }

    #[test]
    fn shadow_invalidation_covers_offset_and_blur_extent() {
        let node = RoundedRectNode {
            id: WidgetId::from_raw(94),
            bounds: rect(20.0, 30.0, 40.0, 20.0),
            corner_radius: 5.0,
            color: Color::rgba(0.2, 0.4, 0.8, 1.0),
            border: None,
            shadow: Some(Shadow {
                offset_x: 4.0,
                offset_y: 6.0,
                blur_radius: 2.0,
                color: Color::rgba(0.0, 0.0, 0.0, 0.4),
            }),
            bevel: None,
            z_index: 0,
        };
        let mut scene = PaintScene::default();
        scene.upsert_rounded_rect(node);

        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());
        assert_eq!(frame.damage, vec![rect(17.0, 29.0, 54.0, 34.0)]);
        assert_eq!(frame.rounded_rects, vec![node]);
    }

    #[test]
    fn local_refresh_repaints_overlapping_nodes_in_z_order() {
        let changed = node(2, rect(10.0, 10.0, 20.0, 20.0), 1);
        let overlap = node(1, rect(15.0, 15.0, 20.0, 20.0), 0);
        let mut scene = PaintScene::default();
        scene.upsert(changed);
        scene.upsert(overlap);
        let _ = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());

        let refresh = RefreshBatch {
            full: false,
            layout_nodes: Vec::new(),
            paint_nodes: vec![changed.id],
        };
        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &refresh);

        assert_eq!(frame.damage, vec![changed.bounds]);
        assert_eq!(frame.commands.len(), 2);
        assert_eq!(frame.commands[0].id, overlap.id);
        assert_eq!(frame.commands[1].id, changed.id);
    }

    #[test]
    fn moving_a_node_damages_both_old_and_new_bounds() {
        let id = WidgetId::from_raw(4);
        let mut scene = PaintScene::default();
        scene.upsert(node(4, rect(0.0, 0.0, 10.0, 10.0), 0));
        let _ = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());

        let moved = node(4, rect(40.0, 0.0, 10.0, 10.0), 0);
        scene.upsert(moved);
        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());

        assert_eq!(frame.damage, vec![rect(0.0, 0.0, 10.0, 10.0), moved.bounds]);
        assert_eq!(frame.commands.len(), 1);
        assert_eq!(frame.commands[0].id, id);
    }

    #[test]
    fn removal_damages_the_old_bounds_and_full_refresh_uses_viewport() {
        let id = WidgetId::from_raw(7);
        let mut scene = PaintScene::default();
        scene.upsert(node(7, rect(5.0, 6.0, 7.0, 8.0), 0));
        let _ = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());
        assert!(scene.remove(id).is_some());

        let frame = scene.take_frame(
            rect(0.0, 0.0, 100.0, 100.0),
            &RefreshBatch {
                full: true,
                layout_nodes: Vec::new(),
                paint_nodes: Vec::new(),
            },
        );

        assert!(frame.full);
        assert_eq!(frame.damage, vec![rect(0.0, 0.0, 100.0, 100.0)]);
        assert!(frame.commands.is_empty());
    }

    #[test]
    fn updating_text_damages_old_and_new_bounds() {
        let id = WidgetId::from_raw(9);
        let mut scene = PaintScene::default();
        let old = TextNode {
            id,
            text: "before".to_owned(),
            bounds: rect(2.0, 3.0, 20.0, 10.0),
            font_size: 12.0,
            line_height: 14.0,
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            z_index: 0,
        };
        scene.upsert_text(old.clone());
        let _ = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());

        let new = TextNode {
            text: "after".to_owned(),
            bounds: rect(40.0, 30.0, 24.0, 10.0),
            ..old
        };
        scene.upsert_text(new.clone());
        let frame = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());

        assert_eq!(frame.damage, vec![old.bounds, new.bounds]);
        assert_eq!(frame.text, vec![new]);
    }

    #[test]
    fn local_refresh_includes_text_nodes() {
        let id = WidgetId::from_raw(11);
        let text = TextNode {
            id,
            text: "refresh".to_owned(),
            bounds: rect(12.0, 15.0, 36.0, 14.0),
            font_size: 12.0,
            line_height: 14.0,
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            z_index: 0,
        };
        let mut scene = PaintScene::default();
        scene.upsert_text(text.clone());
        let _ = scene.take_frame(rect(0.0, 0.0, 100.0, 100.0), &RefreshBatch::default());

        let frame = scene.take_frame(
            rect(0.0, 0.0, 100.0, 100.0),
            &RefreshBatch {
                full: false,
                layout_nodes: Vec::new(),
                paint_nodes: vec![id],
            },
        );

        assert_eq!(frame.damage, vec![text.bounds]);
        assert_eq!(frame.text, vec![text]);
    }
}
