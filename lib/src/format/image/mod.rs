use std::u8;

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
                g: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                b: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
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
                g: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                b: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
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
                g: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                b: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
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
                g: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
                b: ((i * col_1.r as u32 + (3 - i) * col_2.r as u32) / 3) as u8,
            }
        };

        [clo(0), clo(1), clo(2), clo(3)]
    }

    pub fn rows(&self) -> [[RGBColour; 4]; 4] {
        [self.row_1(), self.row_2(), self.row_3(), self.row_4()]
    }
}

#[derive(Debug)]
pub struct DXT1 {
    width: u32,
    height: u32,
    blocks: Vec<DXT1Block>,
}

impl DXT1 {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut rgb_colours = Vec::<RGBColour>::new();

        self.blocks.iter().for_each(|block| {
            rgb_colours.extend_from_slice(block.rows().as_flattened());
        });

        rgb_colours.iter().flat_map(|c| [c.r, c.g, c.b]).collect()
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

#[derive(Debug, Clone, Copy)]
pub struct R5G6B5Colour {
    data: u16,
}

impl From<u16> for R5G6B5Colour {
    fn from(value: u16) -> Self {
        R5G6B5Colour { data: value }
    }
}

impl From<R5G6B5Colour> for RGBColour {
    fn from(colour: R5G6B5Colour) -> Self {
        // In its proper format,
        // RRRRRGGG GGGBBBBB
        //
        // In little endian on disk as a u16,
        // GGGBBBBB RRRRRGGG

        let r_raw = (colour.data >> 3) & 0b11111;
        let g_raw = (colour.data << 3) & 0b111111;
        let b_raw = (colour.data >> 8) & 0b11111;

        RGBColour {
            r: (r_raw * 255 / 31) as u8,
            g: (g_raw * 255 / 31) as u8,
            b: (b_raw * 255 / 31) as u8,
        }
    }
}

impl From<R5G6B5Colour> for RGBAColour {
    fn from(colour: R5G6B5Colour) -> Self {
        RGBColour::from(colour).into()
    }
}

#[derive(Clone)]
pub struct RGBColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<RGBColour> for RGBAColour {
    fn from(colour: RGBColour) -> Self {
        RGBAColour {
            r: colour.r,
            g: colour.g,
            b: colour.b,
            a: u8::MAX,
        }
    }
}

struct RGBAColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
