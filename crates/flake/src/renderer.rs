use std::ops::Range;

use wgpu::util::DeviceExt;

use crate::text_render::TextRenderResources;
use crate::{Border, Color, PaintFrame, Rect, RoundedRectNode, WidgetId};

const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) fill_color: vec4<f32>,
    @location(1) local_position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) corner_radius: f32,
    @location(4) border_width: f32,
    @location(5) border_color: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) fill_color: vec4<f32>,
    @location(2) local_position: vec2<f32>,
    @location(3) size: vec2<f32>,
    @location(4) corner_radius: f32,
    @location(5) border_width: f32,
    @location(6) border_color: vec4<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.fill_color = fill_color;
    output.local_position = local_position;
    output.size = size;
    output.corner_radius = corner_radius;
    output.border_width = border_width;
    output.border_color = border_color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = input.size * 0.5;
    let radius = clamp(input.corner_radius, 0.0, min(half_size.x, half_size.y));
    let q = abs(input.local_position - half_size) - half_size + vec2<f32>(radius);
    let distance = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
    let antialias = max(fwidth(distance), 0.0001);
    let coverage = 1.0 - smoothstep(-antialias, antialias, distance);
    var color = input.fill_color;
    if input.border_width > 0.0 {
        let border_mix = smoothstep(
            -input.border_width - antialias,
            -input.border_width + antialias,
            distance,
        );
        color = mix(input.fill_color, input.border_color, border_mix);
    }
    color.a *= coverage;
    return color;
}
"#;

const PRESENT_SHADER: &str = r#"
@group(0) @binding(0) var source_texture: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(source_texture, vec2<i32>(position.xy), 0);
}
"#;

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
    0 => Float32x2,
    1 => Float32x4,
    2 => Float32x2,
    3 => Float32x2,
    4 => Float32,
    5 => Float32,
    6 => Float32x4
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    InvalidViewport,
    TooManyVertices,
    TextPrepare(String),
    TextRender(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderViewport {
    pub physical_size: [u32; 2],
    pub scale_factor: f32,
}

struct RegionDraw {
    scissor: [u32; 4],
    background_vertices: Range<u32>,
    content_vertices: Range<u32>,
}

#[derive(Clone, Copy)]
enum ShapeRef<'a> {
    Rect(&'a crate::FillRect),
    RoundedRect(&'a RoundedRectNode),
}

impl ShapeRef<'_> {
    fn id(self) -> WidgetId {
        match self {
            Self::Rect(node) => node.id,
            Self::RoundedRect(node) => node.id,
        }
    }

    fn z_index(self) -> i32 {
        match self {
            Self::Rect(node) => node.z_index,
            Self::RoundedRect(node) => node.z_index,
        }
    }
}

/// Basic wgpu renderer for retained rectangle, rounded-rectangle, and text frames.
pub struct WgpuRenderer {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    background_pipeline: wgpu::RenderPipeline,
    present_pipeline: wgpu::RenderPipeline,
    present_bind_group_layout: wgpu::BindGroupLayout,
    text: Option<TextRenderResources>,
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
        let present_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("snow-ui-flake present shader"),
            source: wgpu::ShaderSource::Wgsl(PRESENT_SHADER.into()),
        });
        let present_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("snow-ui-flake present bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });
        let present_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("snow-ui-flake present pipeline layout"),
                bind_group_layouts: &[&present_bind_group_layout],
                immediate_size: 0,
            });
        let present_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("snow-ui-flake present pipeline"),
            layout: Some(&present_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &present_shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &present_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            format,
            pipeline,
            background_pipeline,
            present_pipeline,
            present_bind_group_layout,
            text: None,
        }
    }

    /// Encode a frame into a command buffer for the supplied render target.
    ///
    /// `viewport_size` is the target size in physical pixels. Paint coordinates
    /// are logical pixels and are scaled by `scale_factor` for scissor testing.
    pub fn encode_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        frame: &PaintFrame,
        render_viewport: RenderViewport,
        background: Color,
    ) -> Result<wgpu::CommandBuffer, RenderError> {
        let viewport_size = render_viewport.physical_size;
        let scale_factor = render_viewport.scale_factor;
        if viewport_size[0] == 0
            || viewport_size[1] == 0
            || !scale_factor.is_finite()
            || scale_factor <= 0.0
        {
            return Err(RenderError::InvalidViewport);
        }
        if !frame.full && frame.damage.is_empty() {
            let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("snow-ui-flake empty paint frame"),
            });
            return Ok(encoder.finish());
        }

        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: viewport_size[0] as f32 / scale_factor,
            height: viewport_size[1] as f32 / scale_factor,
        };
        if !frame.text.is_empty() {
            if self.text.is_none() {
                self.text = Some(TextRenderResources::new(device, queue, self.format));
            }
            let text = self.text.as_mut().expect("text resources were initialized");
            text.prepare(device, queue, &frame.text, viewport_size, scale_factor)
                .map_err(RenderError::TextPrepare)?;
        }
        let full_damage = [viewport];
        let damage_regions = if frame.full {
            &full_damage[..]
        } else {
            &frame.damage[..]
        };
        let mut shapes: Vec<_> = frame
            .commands
            .iter()
            .map(ShapeRef::Rect)
            .chain(frame.rounded_rects.iter().map(ShapeRef::RoundedRect))
            .collect();
        shapes.sort_by_key(|shape| (shape.z_index(), shape.id()));

        let mut vertices: Vec<[f32; 16]> = Vec::new();
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
            for shape in &shapes {
                match shape {
                    ShapeRef::Rect(command) if intersects(command.bounds, *damage) => {
                        push_shape(
                            &mut vertices,
                            command.bounds,
                            viewport,
                            command.color,
                            0.0,
                            None,
                        );
                    }
                    ShapeRef::RoundedRect(command) if intersects(command.bounds, *damage) => {
                        push_shape(
                            &mut vertices,
                            command.bounds,
                            viewport,
                            command.color,
                            command.corner_radius,
                            command.border,
                        );
                    }
                    _ => {}
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
        for region in &regions {
            pass.set_scissor_rect(
                region.scissor[0],
                region.scissor[1],
                region.scissor[2],
                region.scissor[3],
            );
            if let Some(buffer) = &buffer {
                pass.set_vertex_buffer(0, buffer.slice(..));
                if !region.background_vertices.is_empty() {
                    pass.set_pipeline(&self.background_pipeline);
                    pass.draw(region.background_vertices.clone(), 0..1);
                }
                if !region.content_vertices.is_empty() {
                    pass.set_pipeline(&self.pipeline);
                    pass.draw(region.content_vertices.clone(), 0..1);
                }
            }
            if !frame.text.is_empty() {
                let text = self.text.as_mut().expect("text resources were initialized");
                text.render(&mut pass).map_err(RenderError::TextRender)?;
            }
        }

        drop(pass);
        Ok(encoder.finish())
    }

    /// Copy a persistent render texture to a newly acquired surface texture.
    pub fn encode_present(
        &self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        background: Color,
    ) -> wgpu::CommandBuffer {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("snow-ui-flake present bind group"),
            layout: &self.present_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source),
            }],
        });
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(to_wgpu_color(background)),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("snow-ui-flake present frame"),
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("snow-ui-flake present pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.present_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        encoder.finish()
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
                array_stride: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
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

fn push_shape(
    vertices: &mut Vec<[f32; 16]>,
    rect: Rect,
    viewport: Rect,
    color: Color,
    corner_radius: f32,
    border: Option<Border>,
) {
    let x0 = rect.x / viewport.width * 2.0 - 1.0;
    let x1 = (rect.x + rect.width) / viewport.width * 2.0 - 1.0;
    let y0 = 1.0 - rect.y / viewport.height * 2.0;
    let y1 = 1.0 - (rect.y + rect.height) / viewport.height * 2.0;
    let corner_radius = if corner_radius.is_finite() {
        corner_radius.max(0.0)
    } else {
        0.0
    };
    let (border_width, border_color) = border.map_or((0.0, Color::TRANSPARENT), |border| {
        let width = if border.width.is_finite() {
            border
                .width
                .max(0.0)
                .min(rect.width.min(rect.height).max(0.0) * 0.5)
        } else {
            0.0
        };
        (width, border.color)
    });
    let fill_color = [color.red, color.green, color.blue, color.alpha];
    let border_color = [
        border_color.red,
        border_color.green,
        border_color.blue,
        border_color.alpha,
    ];
    let vertex = |x, y, local_x, local_y| {
        [
            x,
            y,
            fill_color[0],
            fill_color[1],
            fill_color[2],
            fill_color[3],
            local_x,
            local_y,
            rect.width,
            rect.height,
            corner_radius,
            border_width,
            border_color[0],
            border_color[1],
            border_color[2],
            border_color[3],
        ]
    };
    vertices.extend([
        vertex(x0, y0, 0.0, 0.0),
        vertex(x0, y1, 0.0, rect.height),
        vertex(x1, y0, rect.width, 0.0),
        vertex(x1, y0, rect.width, 0.0),
        vertex(x0, y1, 0.0, rect.height),
        vertex(x1, y1, rect.width, rect.height),
    ]);
}

fn push_rect(vertices: &mut Vec<[f32; 16]>, rect: Rect, viewport: Rect, color: Color) {
    push_shape(vertices, rect, viewport, color, 0.0, None);
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
        assert_eq!(&vertices[0][2..6], &[0.1, 0.2, 0.3, 1.0]);
        assert_eq!(&vertices[0][6..12], &[0.0, 0.0, 50.0, 20.0, 0.0, 0.0]);
    }

    #[test]
    fn rounded_rectangle_vertices_include_radius_and_border() {
        let mut vertices = Vec::new();
        push_shape(
            &mut vertices,
            Rect {
                x: 10.0,
                y: 5.0,
                width: 40.0,
                height: 20.0,
            },
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
            Color::rgba(0.1, 0.2, 0.3, 1.0),
            8.0,
            Some(Border {
                width: 3.0,
                color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            }),
        );

        assert_eq!(vertices.len(), 6);
        assert_eq!(&vertices[0][10..16], &[8.0, 3.0, 1.0, 1.0, 1.0, 1.0]);
    }
}
