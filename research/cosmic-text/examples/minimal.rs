use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};

fn main() {
    let mut font_system = FontSystem::new();

    let metrics = Metrics::new(18.0, 24.0);
    let mut buffer = Buffer::new(&mut font_system, metrics);

    let attrs = Attrs::new().family(Family::SansSerif);

    buffer.set_text(
        "Hello, cosmic-text!\nThis minimal example shows text shaping and layout.",
        &attrs,
        Shaping::Advanced,
        None,
    );

    buffer.shape_until_scroll(&mut font_system, false);

    println!("cosmic-text buffer:");
    println!("  font metrics: {}", metrics);
    println!("  line count: {}", buffer.lines.len());

    for run in buffer.layout_runs() {
        println!(
            "  line {}: rtl={} top={:.1} height={:.1} width={:.1} text={:?} glyphs={}",
            run.line_i,
            run.rtl,
            run.line_top,
            run.line_height,
            run.line_w,
            run.text,
            run.glyphs.len()
        );
    }
}
