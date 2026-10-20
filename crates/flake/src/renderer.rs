use std::ops::Range;

use wgpu::util::DeviceExt;

use crate::{Color, PaintFrame, Rect};

const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.color = color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    InvalidViewport,
    TooManyVertices,
}

struct RegionDraw {
    scissor: [u32; 4],
    background_vertices: Range<u32>,
    content_vertices: Range<u32>,
}

/// Basic wgpu renderer for retained solid-rectangle paint frames.
pub struct WgpuRenderer {
    pipeline: wgpu::RenderPipeline,
    background_pipeline: wgpu::RenderPipeline,
}

impl WgpuRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("snow-ui-flake rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline = create_pipeline(
            device,
            &shader,
            format,
            "snow-ui-flake rectangle pipeline",
            Some(wgpu::BlendState::ALPHA_BLENDING),
        );
        let background_pipeline = create_pipeline(
            device,
            &shader,
            format,
            "snow-ui-flake damage background pipeline",
            None,
        );

        Self {
            pipeline,
            background_pipeline,
        }
    }

    /// Encode a frame into a command buffer for the supplied render target.
    ///
    /// `viewport_size` is the target size in physical pixels. Paint coordinates
    /// are logical pixels and are scaled by `scale_factor` for scissor testing.
    pub fn encode_frame(
        &self,
        device: &wgpu::Device,
        target: &wgpu::TextureView,
        frame: &PaintFrame,
        viewport_size: [u32; 2],
        scale_factor: f32,
        background: Color,
    ) -> Result<wgpu::CommandBuffer, RenderError> {
        if viewport_size[0] == 0
            || viewport_size[1] == 0
            || !scale_factor.is_finite()
            || scale_factor <= 0.0
        {
            return Err(RenderError::InvalidViewport);
        }

        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: viewport_size[0] as f32 / scale_factor,
            height: viewport_size[1] as f32 / scale_factor,
        };
        let full_damage = [viewport];
        let damage_regions = if frame.full {
            &full_damage[..]
        } else {
            &frame.damage[..]
        };
        let mut vertices: Vec<[f32; 6]> = Vec::new();
        let mut regions = Vec::new();

        for damage in damage_regions {
            let Some(scissor) = scissor_rect(*damage, viewport_size, scale_factor) else {
                continue;
            };
            let background_start =
                u32::try_from(vertices.len()).map_err(|_| RenderError::TooManyVertices)?;
            if !frame.full {
                push_rect(&mut vertices, viewport, viewport, background);
            }
            let background_end =
                u32::try_from(vertices.len()).map_err(|_| RenderError::TooManyVertices)?;
            let content_start = background_end;
            for command in &frame.commands {
                if intersects(command.bounds, *damage) {
                    push_rect(&mut vertices, command.bounds, viewport, command.color);
                }
            }
            let end = u32::try_from(vertices.len()).map_err(|_| RenderError::TooManyVertices)?;
            regions.push(RegionDraw {
                scissor,
                background_vertices: background_start..background_end,
                content_vertices: content_start..end,
            });
        }

        let buffer = if vertices.is_empty() {
            None
        } else {
            Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("snow-ui-flake rectangle vertices"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            )
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("snow-ui-flake paint frame"),
        });
        if regions.is_empty() && !frame.full {
            return Ok(encoder.finish());
        }

        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: if frame.full {
                    wgpu::LoadOp::Clear(to_wgpu_color(background))
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("snow-ui-flake paint pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        if let Some(buffer) = &buffer {
            pass.set_vertex_buffer(0, buffer.slice(..));
            for region in &regions {
                pass.set_scissor_rect(
                    region.scissor[0],
                    region.scissor[1],
                    region.scissor[2],
                    region.scissor[3],
                );
                if !region.background_vertices.is_empty() {
                    pass.set_pipeline(&self.background_pipeline);
                    pass.draw(region.background_vertices.clone(), 0..1);
                }
                if !region.content_vertices.is_empty() {
                    pass.set_pipeline(&self.pipeline);
                    pass.draw(region.content_vertices.clone(), 0..1);
                }
            }
        }

        drop(pass);
        Ok(encoder.finish())
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    label: &'static str,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: None,
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &VERTEX_ATTRIBUTES,
            }],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn push_rect(vertices: &mut Vec<[f32; 6]>, rect: Rect, viewport: Rect, color: Color) {
    let x0 = rect.x / viewport.width * 2.0 - 1.0;
    let x1 = (rect.x + rect.width) / viewport.width * 2.0 - 1.0;
    let y0 = 1.0 - rect.y / viewport.height * 2.0;
    let y1 = 1.0 - (rect.y + rect.height) / viewport.height * 2.0;
    let color = [color.red, color.green, color.blue, color.alpha];
    vertices.extend([
        [x0, y0, color[0], color[1], color[2], color[3]],
        [x0, y1, color[0], color[1], color[2], color[3]],
        [x1, y0, color[0], color[1], color[2], color[3]],
        [x1, y0, color[0], color[1], color[2], color[3]],
        [x0, y1, color[0], color[1], color[2], color[3]],
        [x1, y1, color[0], color[1], color[2], color[3]],
    ]);
}

fn scissor_rect(rect: Rect, viewport: [u32; 2], scale_factor: f32) -> Option<[u32; 4]> {
    let left = (rect.x * scale_factor)
        .floor()
        .clamp(0.0, viewport[0] as f32) as u32;
    let top = (rect.y * scale_factor)
        .floor()
        .clamp(0.0, viewport[1] as f32) as u32;
    let right = ((rect.x + rect.width) * scale_factor)
        .ceil()
        .clamp(0.0, viewport[0] as f32) as u32;
    let bottom = ((rect.y + rect.height) * scale_factor)
        .ceil()
        .clamp(0.0, viewport[1] as f32) as u32;
    (right > left && bottom > top).then_some([left, top, right - left, bottom - top])
}

fn intersects(left: Rect, right: Rect) -> bool {
    left.x <= right.x + right.width
        && right.x <= left.x + left.width
        && left.y <= right.y + right.height
        && right.y <= left.y + left.height
}

fn to_wgpu_color(color: Color) -> wgpu::Color {
    wgpu::Color {
        r: color.red as f64,
        g: color.green as f64,
        b: color.blue as f64,
        a: color.alpha as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scissor_rect_scales_and_clips_to_the_physical_viewport() {
        assert_eq!(
            scissor_rect(
                Rect {
                    x: -2.0,
                    y: 3.0,
                    width: 10.0,
                    height: 8.0,
                },
                [100, 80],
                2.0,
            ),
            Some([0, 6, 16, 16])
        );
    }

    #[test]
    fn scissor_rect_discards_damage_outside_the_viewport() {
        assert_eq!(
            scissor_rect(
                Rect {
                    x: 110.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                },
                [100, 80],
                1.0,
            ),
            None
        );
    }

    #[test]
    fn rectangle_vertices_convert_logical_coordinates_to_ndc() {
        let mut vertices = Vec::new();
        push_rect(
            &mut vertices,
            Rect {
                x: 25.0,
                y: 10.0,
                width: 50.0,
                height: 20.0,
            },
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
            Color::rgba(0.1, 0.2, 0.3, 1.0),
        );

        assert_eq!(vertices.len(), 6);
        assert!((vertices[0][0] + 0.5).abs() < f32::EPSILON);
        assert!((vertices[0][1] - 0.6).abs() < f32::EPSILON);
        assert!((vertices[5][0] - 0.5).abs() < f32::EPSILON);
        assert!((vertices[5][1] + 0.2).abs() < f32::EPSILON);
        assert_eq!(&vertices[0][2..], &[0.1, 0.2, 0.3, 1.0]);
    }
}
