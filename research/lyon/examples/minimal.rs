use lyon::math::Point;
use lyon::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};

fn build_triangle_path() -> Path {
    let mut builder = lyon::path::Path::builder();
    builder.begin(Point::new(0.0, 0.0));
    builder.line_to(Point::new(1.0, 0.0));
    builder.line_to(Point::new(0.5, 0.866));
    builder.close();
    builder.build()
}

fn main() {
    let path = build_triangle_path();

    let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    tessellator
        .tessellate_path(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| vertex.position()),
        )
        .expect("Failed to tessellate path");

    println!("Lyon example completed.");
    println!("Vertex count: {}", geometry.vertices.len());
    println!("Index count: {}", geometry.indices.len());
    println!("Triangles: {}", geometry.indices.len() / 3);
    println!(
        "First vertices: {:?}",
        geometry.vertices.iter().take(3).collect::<Vec<_>>()
    );
}
