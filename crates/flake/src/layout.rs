use std::collections::{HashMap, HashSet};

use taffy::{AvailableSpace, NodeId, Size, Style, TaffyTree};

use crate::{RefreshBatch, WidgetId};

/// A platform-neutral layout node used to build a Taffy tree.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: WidgetId,
    pub style: Style,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(id: WidgetId, style: Style) -> Self {
        Self {
            id,
            style,
            children: Vec::new(),
        }
    }

    pub fn with_children(id: WidgetId, style: Style, children: Vec<Self>) -> Self {
        Self {
            id,
            style,
            children,
        }
    }
}

/// Geometry in absolute coordinates relative to the layout root.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LayoutTreeError {
    DuplicateWidgetId(WidgetId),
    UnknownWidget(WidgetId),
    CannotRemoveRoot,
    Taffy(String),
}

/// A Taffy layout tree addressed by stable Snow UI widget identities.
#[derive(Debug)]
pub struct LayoutTree {
    tree: TaffyTree,
    nodes: HashMap<WidgetId, NodeId>,
    parents: HashMap<WidgetId, Option<WidgetId>>,
    root_id: WidgetId,
    root_node: NodeId,
}

impl LayoutTree {
    pub fn new(root: LayoutNode) -> Result<Self, LayoutTreeError> {
        let root_id = root.id;
        let mut result = Self {
            tree: TaffyTree::new(),
            nodes: HashMap::new(),
            parents: HashMap::new(),
            root_id,
            root_node: NodeId::from(0_u64),
        };
        let root_node = result.build_node(&root, None, &mut HashSet::new())?;
        result.root_node = root_node;
        Ok(result)
    }

    pub fn root_id(&self) -> WidgetId {
        self.root_id
    }

    /// Update a node's style. Taffy marks that node and its ancestors dirty.
    pub fn set_style(&mut self, id: WidgetId, style: Style) -> Result<(), LayoutTreeError> {
        let node = self.node(id)?;
        self.tree
            .set_style(node, style)
            .map_err(|error| LayoutTreeError::Taffy(error.to_string()))
    }

    /// Append a new subtree beneath an existing node.
    ///
    /// Widget IDs must be unique across the entire layout tree. The new subtree
    /// is attached only after its IDs have been validated.
    pub fn append_child(
        &mut self,
        parent_id: WidgetId,
        child: LayoutNode,
    ) -> Result<(), LayoutTreeError> {
        let parent_node = self.node(parent_id)?;
        let mut ids = Vec::new();
        collect_ids(&child, &mut ids);
        let mut seen = HashSet::new();
        for id in ids {
            if self.nodes.contains_key(&id) || !seen.insert(id) {
                return Err(LayoutTreeError::DuplicateWidgetId(id));
            }
        }

        let child_id = child.id;
        let child_node = self.build_node(&child, Some(parent_id), &mut HashSet::new())?;
        self.tree
            .add_child(parent_node, child_node)
            .map_err(|error| LayoutTreeError::Taffy(error.to_string()))?;
        debug_assert_eq!(self.parents.get(&child_id), Some(&Some(parent_id)));
        Ok(())
    }

    /// Remove a node and all of its descendants. The layout root cannot be removed.
    /// Returns removed IDs in child-before-parent order so retained paint nodes
    /// can be cleaned up before the next layout synchronization.
    pub fn remove_subtree(&mut self, id: WidgetId) -> Result<Vec<WidgetId>, LayoutTreeError> {
        if id == self.root_id {
            return Err(LayoutTreeError::CannotRemoveRoot);
        }
        let node = self.node(id)?;
        let parent_id = self.parents.get(&id).copied().flatten();
        let mut subtree = Vec::new();
        self.collect_subtree_postorder(id, &mut subtree)?;

        if let Some(parent_id) = parent_id {
            let parent_node = self.node(parent_id)?;
            self.tree
                .remove_child(parent_node, node)
                .map_err(|error| LayoutTreeError::Taffy(error.to_string()))?;
        }
        let removed_ids = subtree.iter().map(|(widget_id, _)| *widget_id).collect();
        for (widget_id, node_id) in subtree {
            self.tree
                .remove(node_id)
                .map_err(|error| LayoutTreeError::Taffy(error.to_string()))?;
            self.nodes.remove(&widget_id);
            self.parents.remove(&widget_id);
        }
        Ok(removed_ids)
    }

    /// Apply layout invalidations; Taffy propagates each dirty mark to ancestors.
    pub fn apply_refresh(&mut self, batch: &RefreshBatch) -> Result<(), LayoutTreeError> {
        if batch.full {
            return self
                .tree
                .mark_dirty(self.root_node)
                .map_err(|error| LayoutTreeError::Taffy(error.to_string()));
        }

        for id in &batch.layout_nodes {
            let node = self.node(*id)?;
            self.tree
                .mark_dirty(node)
                .map_err(|error| LayoutTreeError::Taffy(error.to_string()))?;
        }
        Ok(())
    }

    /// Compute root layout for the available viewport dimensions.
    pub fn compute_layout(&mut self, width: f32, height: f32) -> Result<(), LayoutTreeError> {
        self.tree
            .compute_layout(
                self.root_node,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
            )
            .map_err(|error| LayoutTreeError::Taffy(error.to_string()))
    }

    /// Return a node's computed geometry relative to the layout root.
    pub fn layout(&self, id: WidgetId) -> Result<LayoutRect, LayoutTreeError> {
        let node = self.node(id)?;
        let layout = self
            .tree
            .layout(node)
            .map_err(|error| LayoutTreeError::Taffy(error.to_string()))?;
        let mut x = layout.location.x;
        let mut y = layout.location.y;
        let mut parent = self.parents.get(&id).copied().flatten();
        while let Some(parent_id) = parent {
            let parent_node = self.node(parent_id)?;
            let parent_layout = self
                .tree
                .layout(parent_node)
                .map_err(|error| LayoutTreeError::Taffy(error.to_string()))?;
            x += parent_layout.location.x;
            y += parent_layout.location.y;
            parent = self.parents.get(&parent_id).copied().flatten();
        }

        Ok(LayoutRect {
            x,
            y,
            width: layout.size.width,
            height: layout.size.height,
        })
    }

    fn build_node(
        &mut self,
        layout_node: &LayoutNode,
        parent: Option<WidgetId>,
        seen: &mut HashSet<WidgetId>,
    ) -> Result<NodeId, LayoutTreeError> {
        if !seen.insert(layout_node.id) {
            return Err(LayoutTreeError::DuplicateWidgetId(layout_node.id));
        }

        let child_nodes = layout_node
            .children
            .iter()
            .map(|child| self.build_node(child, Some(layout_node.id), seen))
            .collect::<Result<Vec<_>, _>>()?;
        let node = if child_nodes.is_empty() {
            self.tree.new_leaf(layout_node.style.clone())
        } else {
            self.tree
                .new_with_children(layout_node.style.clone(), &child_nodes)
        }
        .map_err(|error| LayoutTreeError::Taffy(error.to_string()))?;

        self.nodes.insert(layout_node.id, node);
        self.parents.insert(layout_node.id, parent);
        Ok(node)
    }

    fn node(&self, id: WidgetId) -> Result<NodeId, LayoutTreeError> {
        self.nodes
            .get(&id)
            .copied()
            .ok_or(LayoutTreeError::UnknownWidget(id))
    }

    fn collect_subtree_postorder(
        &self,
        id: WidgetId,
        output: &mut Vec<(WidgetId, NodeId)>,
    ) -> Result<(), LayoutTreeError> {
        for (child_id, parent_id) in &self.parents {
            if *parent_id == Some(id) {
                self.collect_subtree_postorder(*child_id, output)?;
            }
        }
        output.push((id, self.node(id)?));
        Ok(())
    }
}

fn collect_ids(node: &LayoutNode, ids: &mut Vec<WidgetId>) {
    ids.push(node.id);
    for child in &node.children {
        collect_ids(child, ids);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::{Dimension, Display, FlexDirection};

    fn fixed_style(width: f32, height: f32) -> Style {
        Style {
            size: Size {
                width: Dimension::length(width),
                height: Dimension::length(height),
            },
            ..Style::default()
        }
    }

    #[test]
    fn computes_and_accumulates_nested_layout_coordinates() {
        let root_id = WidgetId::from_raw(1);
        let row_id = WidgetId::from_raw(2);
        let leaf_id = WidgetId::from_raw(3);
        let row = LayoutNode::with_children(
            row_id,
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: Dimension::length(200.0),
                    height: Dimension::length(60.0),
                },
                ..Style::default()
            },
            vec![
                LayoutNode::new(WidgetId::from_raw(4), fixed_style(10.0, 20.0)),
                LayoutNode::new(leaf_id, fixed_style(40.0, 20.0)),
            ],
        );
        let root = LayoutNode::with_children(
            root_id,
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: Dimension::length(300.0),
                    height: Dimension::length(100.0),
                },
                ..Style::default()
            },
            vec![
                LayoutNode::new(WidgetId::from_raw(5), fixed_style(5.0, 20.0)),
                row,
            ],
        );
        let mut tree = LayoutTree::new(root).unwrap();

        tree.compute_layout(300.0, 100.0).unwrap();

        assert_eq!(
            tree.layout(leaf_id).unwrap(),
            LayoutRect {
                x: 15.0,
                y: 0.0,
                width: 40.0,
                height: 20.0,
            }
        );
    }

    #[test]
    fn updates_one_node_and_recomputes_its_layout_path() {
        let root_id = WidgetId::from_raw(1);
        let child_id = WidgetId::from_raw(2);
        let root = LayoutNode::with_children(
            root_id,
            Style {
                display: Display::Flex,
                size: Size {
                    width: Dimension::length(100.0),
                    height: Dimension::length(100.0),
                },
                ..Style::default()
            },
            vec![LayoutNode::new(child_id, fixed_style(20.0, 20.0))],
        );
        let mut tree = LayoutTree::new(root).unwrap();
        tree.compute_layout(100.0, 100.0).unwrap();

        tree.set_style(child_id, fixed_style(60.0, 30.0)).unwrap();
        let refresh = crate::RefreshHandle::default();
        refresh.invalidate_layout_path(&[child_id, root_id]);
        tree.apply_refresh(&refresh.take_pending()).unwrap();
        tree.compute_layout(100.0, 100.0).unwrap();

        assert_eq!(tree.layout(child_id).unwrap().width, 60.0);
        assert_eq!(tree.layout(child_id).unwrap().height, 30.0);
    }

    #[test]
    fn rejects_duplicate_widget_ids() {
        let duplicate = WidgetId::from_raw(2);
        let root = LayoutNode::with_children(
            WidgetId::from_raw(1),
            Style::default(),
            vec![
                LayoutNode::new(duplicate, Style::default()),
                LayoutNode::new(duplicate, Style::default()),
            ],
        );

        assert_eq!(
            LayoutTree::new(root).unwrap_err(),
            LayoutTreeError::DuplicateWidgetId(duplicate)
        );
    }

    #[test]
    fn appends_and_removes_dynamic_subtrees() {
        let root_id = WidgetId::from_raw(20);
        let child_id = WidgetId::from_raw(21);
        let nested_id = WidgetId::from_raw(22);
        let mut tree =
            LayoutTree::new(LayoutNode::new(root_id, fixed_style(100.0, 100.0))).unwrap();
        tree.append_child(
            root_id,
            LayoutNode::with_children(
                child_id,
                fixed_style(40.0, 40.0),
                vec![LayoutNode::new(nested_id, fixed_style(10.0, 10.0))],
            ),
        )
        .unwrap();
        tree.compute_layout(100.0, 100.0).unwrap();
        assert_eq!(tree.layout(nested_id).unwrap().width, 10.0);

        assert_eq!(
            tree.remove_subtree(child_id).unwrap(),
            vec![nested_id, child_id]
        );
        assert_eq!(
            tree.layout(child_id),
            Err(LayoutTreeError::UnknownWidget(child_id))
        );
        assert_eq!(
            tree.layout(nested_id),
            Err(LayoutTreeError::UnknownWidget(nested_id))
        );
        tree.compute_layout(100.0, 100.0).unwrap();
    }

    #[test]
    fn rejects_duplicate_ids_when_appending_without_mutating_tree() {
        let root_id = WidgetId::from_raw(30);
        let child_id = WidgetId::from_raw(31);
        let mut tree = LayoutTree::new(LayoutNode::with_children(
            root_id,
            fixed_style(100.0, 100.0),
            vec![LayoutNode::new(child_id, fixed_style(20.0, 20.0))],
        ))
        .unwrap();

        assert_eq!(
            tree.append_child(root_id, LayoutNode::new(child_id, fixed_style(10.0, 10.0))),
            Err(LayoutTreeError::DuplicateWidgetId(child_id))
        );
        assert_eq!(tree.layout(child_id).unwrap().width, 0.0);
    }

    #[test]
    fn refuses_to_remove_the_layout_root() {
        let root_id = WidgetId::from_raw(40);
        let mut tree = LayoutTree::new(LayoutNode::new(root_id, Style::default())).unwrap();
        assert_eq!(
            tree.remove_subtree(root_id),
            Err(LayoutTreeError::CannotRemoveRoot)
        );
        assert_eq!(tree.root_id(), root_id);
    }
}
