use std::collections::{BTreeMap, VecDeque};

use glyphon::{Attrs, Buffer as TextBuffer, Family, FontSystem, Metrics, Shaping};

use crate::{TextNode, WidgetId};

const MAX_CACHED_TEXT_BUFFERS: usize = 128;

pub(crate) struct TextBufferCache {
    buffers: BTreeMap<WidgetId, CachedTextBuffer>,
    order: VecDeque<WidgetId>,
}

struct CachedTextBuffer {
    key: TextBufferKey,
    buffer: TextBuffer,
}

struct TextBufferKey {
    text: String,
    width: f32,
    height: f32,
    font_size: f32,
    line_height: f32,
}

impl TextBufferKey {
    fn matches_node(&self, node: &TextNode) -> bool {
        self.text == node.text
            && self.width == node.bounds.width.max(0.0)
            && self.height == node.bounds.height.max(0.0)
            && self.font_size == node.font_size
            && self.line_height == node.line_height
    }

    fn from_node(node: &TextNode) -> Self {
        Self {
            text: node.text.clone(),
            width: node.bounds.width.max(0.0),
            height: node.bounds.height.max(0.0),
            font_size: node.font_size,
            line_height: node.line_height,
        }
    }
}

impl TextBufferCache {
    pub(crate) fn new() -> Self {
        Self {
            buffers: BTreeMap::new(),
            order: VecDeque::new(),
        }
    }

    pub(crate) fn ensure_buffer(&mut self, font_system: &mut FontSystem, node: &TextNode) {
        let needs_update = self
            .buffers
            .get(&node.id)
            .is_none_or(|cached| !cached.key.matches_node(node));
        if needs_update {
            let key = TextBufferKey::from_node(node);
            let mut buffer =
                TextBuffer::new(font_system, Metrics::new(node.font_size, node.line_height));
            buffer.set_size(
                font_system,
                Some(node.bounds.width.max(0.0)),
                Some(node.bounds.height.max(0.0)),
            );
            buffer.set_text(
                font_system,
                &node.text,
                &Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
                None,
            );
            buffer.shape_until_scroll(font_system, false);
            self.buffers
                .insert(node.id, CachedTextBuffer { key, buffer });
        }

        if let Some(position) = self.order.iter().position(|id| *id == node.id) {
            self.order.remove(position);
        }
        self.order.push_back(node.id);
    }

    pub(crate) fn get(&self, id: WidgetId) -> &TextBuffer {
        &self
            .buffers
            .get(&id)
            .expect("buffer was just cached")
            .buffer
    }

    pub(crate) fn trim(&mut self) {
        while self.buffers.len() > MAX_CACHED_TEXT_BUFFERS {
            if let Some(expired) = self.order.pop_front() {
                self.buffers.remove(&expired);
            }
        }
    }
}
