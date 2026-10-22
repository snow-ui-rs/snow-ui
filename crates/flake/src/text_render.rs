use glyphon::{
    Cache, FontSystem, Resolution, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer,
    Viewport,
};

use crate::text_cache::TextBufferCache;
use crate::{Color, TextNode};

pub(crate) struct TextRenderResources {
    font_system: FontSystem,
    swash_cache: SwashCache,
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
    buffers: TextBufferCache,
}

impl TextRenderResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        Self {
            font_system,
            swash_cache,
            _cache: cache,
            viewport,
            atlas,
            renderer,
            buffers: TextBufferCache::new(),
        }
    }

    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        nodes: &[TextNode],
        viewport_size: [u32; 2],
        scale_factor: f32,
    ) -> Result<(), ()> {
        self.viewport.update(
            queue,
            Resolution {
                width: viewport_size[0],
                height: viewport_size[1],
            },
        );
        for node in nodes {
            self.buffers.ensure_buffer(&mut self.font_system, node);
        }
        let text_areas: Vec<_> = nodes
            .iter()
            .map(|node| TextArea {
                buffer: self.buffers.get(node.id),
                left: node.bounds.x * scale_factor,
                top: node.bounds.y * scale_factor,
                scale: scale_factor,
                bounds: TextBounds {
                    left: (node.bounds.x * scale_factor).max(0.0) as i32,
                    top: (node.bounds.y * scale_factor).max(0.0) as i32,
                    right: ((node.bounds.x + node.bounds.width) * scale_factor)
                        .min(viewport_size[0] as f32) as i32,
                    bottom: ((node.bounds.y + node.bounds.height) * scale_factor)
                        .min(viewport_size[1] as f32) as i32,
                },
                default_color: glyphon_color(node.color),
                custom_glyphs: &[],
            })
            .collect();
        let result = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        );
        self.buffers.trim();
        result.map_err(|_| ())
    }

    pub(crate) fn render(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<(), ()> {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|_| ())
    }
}

fn glyphon_color(color: Color) -> glyphon::Color {
    glyphon::Color::rgba(
        (color.red.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.green.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}
