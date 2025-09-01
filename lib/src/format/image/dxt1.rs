use crate::format::image::types::{R5G6B5Colour, RGBColour};

#[derive(Debug)]
pub struct DXT1 {
    width: u32,
    height: u32,
    blocks: Vec<DXT1Block>,
}

#[derive(Debug)]
pub struct DXT1Block {
    colour_1: R5G6B5Colour,
    colour_2: R5G6B5Colour,

    // On disk          // Desired
    // 03 13 23 33      00 10 20 30
    // 02 12 22 32      01 11 21 31
    // 01 11 21 31      02 12 22 32
    // 00 10 20 30      03 13 23 33
    indices: u32,
}

impl DXT1Block {
    pub fn from_bytes(bytes: &[u8]) -> Result<DXT1Block, std::io::Error> {
        if bytes.len() < size_of::<DXT1Block>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not enough input bytes to form a block.",
            ));
        }

        Ok(DXT1Block {
            colour_1: u16::from_le_bytes(bytes[0..2].try_into().unwrap()).into(),
            colour_2: u16::from_le_bytes(bytes[2..4].try_into().unwrap()).into(),
            indices: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        })
    }

    // Row-order block in the human order
    pub fn row_1(&self) -> [RGBColour; 4] {
        let clo = |col| -> RGBColour {
            let i: u32 = (self.indices << (2 * (col + 1))) & 0b11u32;

            let col_1: RGBColour = self.colour_1.into();
            let col_2: RGBColour = self.colour_2.into();

            RGBColour {
                r: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                g: ((i * col_1.g as u32 + (3 - i) * col_2.g as u32) / 3) as u8,
                b: ((i * col_1.b as u32 + (3 - i) * col_2.b as u32) / 3) as u8,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn row_2(&self) -> [RGBColour; 4] {
        let clo = |col| -> RGBColour {
            let i: u32 = (self.indices >> 8 << (2 * (col + 1))) & 0b11u32;

            let col_1: RGBColour = self.colour_1.into();
            let col_2: RGBColour = self.colour_2.into();

            RGBColour {
                r: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                g: ((i * col_1.g as u32 + (3 - i) * col_2.g as u32) / 3) as u8,
                b: ((i * col_1.b as u32 + (3 - i) * col_2.b as u32) / 3) as u8,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn row_3(&self) -> [RGBColour; 4] {
        let clo = |col| -> RGBColour {
            let i: u32 = (self.indices >> 16 << (2 * (col + 1))) & 0b11u32;

            let col_1: RGBColour = self.colour_1.into();
            let col_2: RGBColour = self.colour_2.into();

            RGBColour {
                r: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                g: ((i * col_1.g as u32 + (3 - i) * col_2.g as u32) / 3) as u8,
                b: ((i * col_1.b as u32 + (3 - i) * col_2.b as u32) / 3) as u8,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn row_4(&self) -> [RGBColour; 4] {
        let clo = |col| -> RGBColour {
            let i: u32 = (self.indices >> 24 << (2 * (col + 1))) & 0b11u32;

            let col_1: RGBColour = self.colour_1.into();
            let col_2: RGBColour = self.colour_2.into();

            RGBColour {
                r: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                g: ((i * col_1.g as u32 + (3 - i) * col_2.g as u32) / 3) as u8,
                b: ((i * col_1.b as u32 + (3 - i) * col_2.b as u32) / 3) as u8,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn rows(&self) -> [[RGBColour; 4]; 4] {
        [self.row_1(), self.row_2(), self.row_3(), self.row_4()]
    }
}

impl DXT1 {
    pub fn as_rgb_bytes(&self) -> Vec<u8> {
        self.as_rgb().iter().flat_map(|c| [c.r, c.g, c.b]).collect()
    }

    pub fn as_rgba_bytes(&self) -> Vec<u8> {
        self.as_rgb()
            .iter()
            .flat_map(|c| [c.r, c.g, c.b, u8::MAX])
            .collect()
    }

    pub fn as_rgb(&self) -> Vec<RGBColour> {
        let blocks_x = self.height.div_ceil(4) as usize;
        let blocks_y = self.width.div_ceil(4) as usize;

        let mut rgb_colours = Vec::<RGBColour>::new();

        let width_stride = (self.height / 4) as usize;

        let mut vec1 = Vec::<RGBColour>::new();
        let mut vec2 = Vec::<RGBColour>::new();
        let mut vec3 = Vec::<RGBColour>::new();
        let mut vec4 = Vec::<RGBColour>::new();

        for outer_row in 0..blocks_y {
            vec1.clear();
            vec2.clear();
            vec3.clear();
            vec4.clear();

            for i in 0..blocks_x {
                let block = &self.blocks[outer_row * width_stride + i];

                vec1.extend_from_slice(&block.row_1());
                vec2.extend_from_slice(&block.row_2());
                vec3.extend_from_slice(&block.row_3());
                vec4.extend_from_slice(&block.row_4());
            }

            rgb_colours.extend_from_slice(&vec1);
            rgb_colours.extend_from_slice(&vec2);
            rgb_colours.extend_from_slice(&vec3);
            rgb_colours.extend_from_slice(&vec4);
        }
        rgb_colours
    }

    pub fn from_bytes(bytes: &[u8], width: u32, height: u32) -> Result<DXT1, std::io::Error> {
        if width == 0 && height == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Both width and height are 0.",
            ));
        }

        let pixel_count = width * height;

        let bytes_required = pixel_count / 2; // DXT1 is 16 pixels per 8 bytes, so /16 x8 gives
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
            blocks.push(DXT1Block::from_bytes(
                &bytes[(i * size_of::<DXT1Block>())..],
            )?);
        }

        // let blocks = ;

        Ok(DXT1 {
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
