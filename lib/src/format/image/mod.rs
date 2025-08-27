use crate::format::image::{dxt1::DXT1, dxt2::DXT2};

pub mod dxt1;
pub mod dxt2;
pub mod types;

pub enum Image {
    DXT1(DXT1),
    DXT2(DXT2),
}
