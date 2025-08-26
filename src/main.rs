use std::{fs::File, io::BufWriter, path::Path};

use anyxplore::format::image::DXT1;

fn main() {
    let filename = std::env::args().nth(1).unwrap_or_default();
    let width = std::env::args()
        .nth(2)
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or(0);
    let height = std::env::args()
        .nth(3)
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or(0);

    if filename.is_empty() {
        eprintln!("Unable to find input file.");
        std::process::exit(1);
    }

    let bytes = match std::fs::read(filename) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Unable to read file.\nError: {}", e);
            std::process::exit(1);
        }
    };

    let dxt = DXT1::from_bytes(bytes.as_ref(), width, height).expect("Unable to get dxt1.");

    let bytes = dxt.as_bytes();

    let path = Path::new(r"./out.png");
    let file = File::create(path).unwrap();
    let w = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, dxt.width(), dxt.height()); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    /*
        encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2)); // 1.0 / 2.2, unscaled, but rounded
        let source_chromaticities = png::SourceChromaticities::new(
            // Using unscaled instantiation here
            (0.31270, 0.32900),
            (0.64000, 0.33000),
            (0.30000, 0.60000),
            (0.15000, 0.06000),
        );
        encoder.set_source_chromaticities(source_chromaticities);
    */

    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&bytes).unwrap();
    writer.finish().expect("Unable to close writer.");
}
