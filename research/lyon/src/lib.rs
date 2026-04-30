//! Shared utilities for Lyon research examples.

use lyon::math::Point;
use lyon::path::Path;

pub fn build_triangle_path() -> Path {
    let mut builder = Path::builder();
    builder.begin(Point::new(0.0, 0.0));
    builder.line_to(Point::new(1.0, 0.0));
    builder.line_to(Point::new(0.5, 0.866));
    builder.close();
    builder.build()
}
