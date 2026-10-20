use std::collections::BTreeMap;

use crate::{RefreshBatch, WidgetId};

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
}

/// Retained paint nodes with damage tracking for local redraws.
#[derive(Debug, Default)]
pub struct PaintScene {
    nodes: BTreeMap<WidgetId, PaintNode>,
    pending_damage: Vec<Rect>,
}

impl PaintScene {
    /// Add or update a rectangle, recording both old and new bounds as damage.
    pub fn upsert(&mut self, node: PaintNode) {
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

    /// Remove a node and mark its former bounds for clearing.
    pub fn remove(&mut self, id: WidgetId) -> Option<PaintNode> {
        let removed = self.nodes.remove(&id)?;
        self.add_damage(removed.bounds);
        Some(removed)
    }

    /// Build a frame from accumulated geometry changes and paint invalidations.
    /// Commands include every retained node intersecting a damage region, sorted
    /// by `(z_index, WidgetId)` so overlapping content is repainted correctly.
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

        PaintFrame {
            full,
            damage,
            commands,
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
}
