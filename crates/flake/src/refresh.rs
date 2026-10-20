use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use crate::WidgetId;

#[derive(Default)]
struct PendingRefresh {
    full: bool,
    layout: BTreeSet<WidgetId>,
    paint: BTreeSet<WidgetId>,
}

/// Thread-safe handle for coalescing local and full refresh requests.
#[derive(Clone, Default)]
pub struct RefreshHandle {
    pending: Arc<Mutex<PendingRefresh>>,
}

/// A drained set of refresh work, with stable ordering and duplicate IDs removed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RefreshBatch {
    pub full: bool,
    pub layout_nodes: Vec<WidgetId>,
    pub paint_nodes: Vec<WidgetId>,
}

impl RefreshHandle {
    /// Mark a widget and its ancestors for layout, and the widget for repaint.
    pub fn invalidate_layout(&self, widget_id: WidgetId) {
        self.invalidate_layout_path(&[widget_id]);
    }

    /// Mark the changed widget and its ancestors for layout. `path` is ordered
    /// from the changed widget to the root; the layout pass determines any other
    /// widgets whose geometry or paint is affected by the change.
    pub fn invalidate_layout_path(&self, path: &[WidgetId]) {
        let Some(widget_id) = path.first().copied() else {
            return;
        };
        let mut pending = self.lock_pending();
        if !pending.full {
            pending.layout.extend(path.iter().copied());
            pending.paint.insert(widget_id);
        }
    }

    /// Mark only a widget's visual content for repaint.
    pub fn invalidate_paint(&self, widget_id: WidgetId) {
        let mut pending = self.lock_pending();
        if !pending.full {
            pending.paint.insert(widget_id);
        }
    }

    /// Discard narrower pending requests and schedule a complete refresh.
    pub fn invalidate_all(&self) {
        let mut pending = self.lock_pending();
        pending.full = true;
        pending.layout.clear();
        pending.paint.clear();
    }

    /// Take all pending work, resetting the handle for the next frame.
    pub fn take_pending(&self) -> RefreshBatch {
        let mut pending = self.lock_pending();
        let batch = RefreshBatch {
            full: pending.full,
            layout_nodes: std::mem::take(&mut pending.layout).into_iter().collect(),
            paint_nodes: std::mem::take(&mut pending.paint).into_iter().collect(),
        };
        pending.full = false;
        batch
    }

    fn lock_pending(&self) -> std::sync::MutexGuard<'_, PendingRefresh> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_invalidations_are_coalesced() {
        let refresh = RefreshHandle::default();
        let first = WidgetId::from_raw(4);
        let second = WidgetId::from_raw(9);

        refresh.invalidate_paint(first);
        refresh.invalidate_paint(first);
        refresh.invalidate_layout(second);
        refresh.invalidate_layout(second);

        assert_eq!(
            refresh.take_pending(),
            RefreshBatch {
                full: false,
                layout_nodes: vec![second],
                paint_nodes: vec![first, second],
            }
        );
        assert_eq!(refresh.take_pending(), RefreshBatch::default());
    }

    #[test]
    fn full_refresh_supersedes_local_invalidations() {
        let refresh = RefreshHandle::default();
        let widget = WidgetId::from_raw(4);

        refresh.invalidate_layout(widget);
        refresh.invalidate_all();
        refresh.invalidate_paint(widget);

        assert_eq!(
            refresh.take_pending(),
            RefreshBatch {
                full: true,
                layout_nodes: Vec::new(),
                paint_nodes: Vec::new(),
            }
        );
    }

    #[test]
    fn layout_invalidation_marks_the_ancestor_path() {
        let refresh = RefreshHandle::default();
        let widget = WidgetId::from_raw(4);
        let parent = WidgetId::from_raw(8);
        let root = WidgetId::from_raw(12);

        refresh.invalidate_layout_path(&[widget, parent, root]);

        assert_eq!(
            refresh.take_pending(),
            RefreshBatch {
                full: false,
                layout_nodes: vec![widget, parent, root],
                paint_nodes: vec![widget],
            }
        );
    }
}
