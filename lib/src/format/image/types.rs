#[derive(Debug, Clone, Copy)]
pub struct R5G6B5Colour {
    data: u16,
}

#[derive(Clone)]
pub struct RGBColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone)]
pub struct RGBAColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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

impl From<R5G6B5Colour> for RGBAColour {
    fn from(colour: R5G6B5Colour) -> Self {
        RGBColour::from(colour).into()
    }
}
