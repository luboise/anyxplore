use crate::format::image::types::{R5G6B5Colour, RGBAColour};

#[derive(Debug)]
pub struct DXT2 {
    width: u32,
    height: u32,
    blocks: Vec<DXT2Block>,
}

#[derive(Debug)]
pub struct DXT2Block {
    alpha_data: [u8; 8],

    colour_1: R5G6B5Colour,
    colour_2: R5G6B5Colour,

    // On disk          // Desired
    // 03 13 23 33      00 10 20 30
    // 02 12 22 32      01 11 21 31
    // 01 11 21 31      02 12 22 32
    // 00 10 20 30      03 13 23 33
    indices: u32,
}

impl DXT2Block {
    pub fn from_bytes(bytes: &[u8]) -> Result<DXT2Block, std::io::Error> {
        if bytes.len() < size_of::<DXT2Block>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not enough input bytes to form a block.",
            ));
        }

        Ok(DXT2Block {
            // TODO: Remove this expect
            alpha_data: bytes[0..8].try_into().expect("Unable to convert"),
            colour_1: u16::from_le_bytes(bytes[8..10].try_into().unwrap()).into(),
            colour_2: u16::from_le_bytes(bytes[10..12].try_into().unwrap()).into(),
            indices: u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
        })
    }

    // Row-order block in the human order
    pub fn row_1_rgba(&self) -> [RGBAColour; 4] {
        let clo = |col| -> RGBAColour {
            let i: u32 = (self.indices << (2 * (col + 1))) & 0b11u32;

            let col_1: RGBAColour = self.colour_1.into();
            let col_2: RGBAColour = self.colour_2.into();

            RGBAColour {
                r: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                g: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                b: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                a: u8::MAX,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn row_2_rgba(&self) -> [RGBAColour; 4] {
        let clo = |col| -> RGBAColour {
            let i: u32 = (self.indices >> 8 << (2 * (col + 1))) & 0b11u32;

            let col_1: RGBAColour = self.colour_1.into();
            let col_2: RGBAColour = self.colour_2.into();

            RGBAColour {
                r: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                g: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                b: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                a: u8::MAX,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn row_3_rgba(&self) -> [RGBAColour; 4] {
        let clo = |col| -> RGBAColour {
            let i: u32 = (self.indices >> 16 << (2 * (col + 1))) & 0b11u32;

            let col_1: RGBAColour = self.colour_1.into();
            let col_2: RGBAColour = self.colour_2.into();

            RGBAColour {
                r: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                g: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                b: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                a: u8::MAX,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn row_4_rgba(&self) -> [RGBAColour; 4] {
        let clo = |col| -> RGBAColour {
            let i: u32 = (self.indices >> 24 << (2 * (col + 1))) & 0b11u32;

            let col_1: RGBAColour = self.colour_1.into();
            let col_2: RGBAColour = self.colour_2.into();

            RGBAColour {
                r: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                g: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                b: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                a: u8::MAX,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn rows(&self) -> [[RGBAColour; 4]; 4] {
        [
            self.row_1_rgba(),
            self.row_2_rgba(),
            self.row_3_rgba(),
            self.row_4_rgba(),
        ]
    }
}

impl DXT2 {
    pub fn as_rgba_bytes(&self) -> Vec<u8> {
        let mut rgba_colours = Vec::<RGBAColour>::new();

        let width_stride = (self.height / 4) as usize;

        let mut vec1 = Vec::<RGBAColour>::new();
        let mut vec2 = Vec::<RGBAColour>::new();
        let mut vec3 = Vec::<RGBAColour>::new();
        let mut vec4 = Vec::<RGBAColour>::new();

        for outer_row in 0..(self.width / 4) as usize {
            vec1.clear();
            vec2.clear();
            vec3.clear();
            vec4.clear();

            for i in 0..width_stride {
                let block = &self.blocks[outer_row * width_stride + i];

                vec1.extend_from_slice(&block.row_1_rgba());
                vec2.extend_from_slice(&block.row_2_rgba());
                vec3.extend_from_slice(&block.row_3_rgba());
                vec4.extend_from_slice(&block.row_4_rgba());
            }

            rgba_colours.extend_from_slice(&vec1);
            rgba_colours.extend_from_slice(&vec2);
            rgba_colours.extend_from_slice(&vec3);
            rgba_colours.extend_from_slice(&vec4);
        }

        rgba_colours
            .iter()
            .flat_map(|c| [c.r, c.g, c.b, c.a])
            .collect()
    }

    pub fn from_bytes(bytes: &[u8], width: u32, height: u32) -> Result<DXT2, std::io::Error> {
        if width == 0 && height == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Both width and height are 0.",
            ));
        }

        let pixel_count = width * height;

        let bytes_required = pixel_count / 2; // DXT2 is 16 pixels per 8 bytes, so /16 x8 gives
        // number of bytes required

        if bytes.len() < bytes_required as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "The byte slice supplied is not large enough ({} x {} = {} pixels, need {}, only have {})",
                    width,
                    height,
                    pixel_count,
                    bytes_required,
                    bytes.len()
                ),
            ));
        }

        let block_count = bytes_required / 8;

        let mut blocks = Vec::new();

        for i in 0..block_count as usize {
            blocks.push(DXT2Block::from_bytes(
                &bytes[(i * size_of::<DXT2Block>())..],
            )?);
        }

        // let blocks = ;

        Ok(DXT2 {
            width,
            height,
            blocks,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
