use bnl::asset::texture::Texture;
use fltk::{frame, image::RgbImage, prelude::*};

pub trait Viewable {
    fn create_viewer<T: GroupExt>(&self, widget_parent: &mut T);
}

pub trait Editable: Viewable {
    fn create_editor<T: GroupExt>(&mut self, widget_parent: &mut T);
}

impl Viewable for Texture {
    fn create_viewer<T: GroupExt>(&self, widget_parent: &mut T) {
        if let Ok(rgba) = self.to_rgba_image() {
            if let Ok(mut img) = unsafe {
                RgbImage::from_data(
                    rgba.bytes(),
                    rgba.width() as i32,
                    rgba.height() as i32,
                    fltk::enums::ColorDepth::Rgba8,
                )
            } {
                println!("Setting image.");

                let mut frame = frame::Frame::default().size_of(widget_parent);
                img.scale(frame.width(), frame.height(), true, true);
                frame.set_image(Some(img));

                widget_parent.add(&frame);
            }
        }
    }
}

impl Editable for Texture {
    fn create_editor<T: GroupExt>(&mut self, widget_parent: &mut T) {
        todo!();
    }
}
