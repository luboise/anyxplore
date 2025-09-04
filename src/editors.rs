use bnl::asset::{
    Asset,
    texture::{Texture, TextureDescriptor},
};
use fltk::{enums, frame, group, image::RgbImage, input, prelude::*};

#[derive(Debug)]
pub enum CreationFailure {
    PartialFailure(String),
    CompleteFailure(String),
}

pub trait Viewable {
    fn create_viewer<T: GroupExt>(&self, widget_parent: &mut T) -> Result<(), CreationFailure>;
}

pub trait Editable: Viewable {
    fn create_editor<T: GroupExt>(&mut self, widget_parent: &mut T) -> Result<(), CreationFailure>;
}

pub(crate) fn set_viewer_grid_title(
    grid: &mut group::Grid,
    label_text: &str,
) -> Result<(), FltkError> {
    let mut title = frame::Frame::default().with_label(label_text);
    title.set_frame(enums::FrameType::FlatBox);
    title.set_color(enums::Color::Red);
    title.set_label_color(enums::Color::White);

    grid.set_widget(&mut title, 0, 0..2)
}

pub(crate) fn set_viewer_grid_row<T: std::fmt::Debug>(
    grid: &mut group::Grid,
    row: usize,
    label_text: &str,
    value: T,
) -> Result<(), FltkError> {
    let label = &mut frame::Frame::default().with_label(label_text);
    grid.set_widget(label, row, 0)?;

    let input = &mut input::Input::default();

    input.set_readonly(true);
    input.set_value(&format!("{:?}", value));

    grid.set_widget(input, row, 1)?;

    Ok(())
}

impl Viewable for TextureDescriptor {
    fn create_viewer<T: GroupExt>(&self, widget_parent: &mut T) -> Result<(), CreationFailure> {
        let mut grid = group::Grid::default().with_size(300, 400);
        grid.show_grid(false);

        grid.set_layout(10, 2);

        let mut create_grid = || -> Result<(), FltkError> {
            // 1 row for title

            // 3 rows
            set_viewer_grid_title(&mut grid, "Texture")?;

            set_viewer_grid_row(&mut grid, 1, "Format", self.format())?;
            set_viewer_grid_row(&mut grid, 2, "Header Size", self.header_size())?;
            set_viewer_grid_row(&mut grid, 3, "Width", self.width())?;
            set_viewer_grid_row(&mut grid, 4, "Height", self.height())?;
            set_viewer_grid_row(&mut grid, 5, "Flags", self.flags())?;
            set_viewer_grid_row(&mut grid, 6, "Unknown3a", self.unknown_3a())?;
            set_viewer_grid_row(&mut grid, 7, "Resource Offset", self.texture_offset())?;
            set_viewer_grid_row(&mut grid, 8, "Resource Size", self.texture_size())?;

            Ok(())
        };

        create_grid().map_err(|e| {
            grid.end();
            CreationFailure::CompleteFailure(e.to_string())
        })?;

        grid.end();
        widget_parent.add(&grid);

        Ok(())
    }
}

impl Viewable for Texture {
    fn create_viewer<T: GroupExt>(&self, widget_parent: &mut T) -> Result<(), CreationFailure> {
        self.descriptor().create_viewer(widget_parent)?;

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

                let mut frame = frame::Frame::default().with_size(widget_parent.width(), 500);
                img.scale(frame.width(), frame.height(), true, true);
                frame.set_image(Some(img));

                widget_parent.add(&frame);
            }
        } else {
            widget_parent.add(&frame::Frame::default().with_label("Error creating image view."));
        }

        Ok(())
    }
}

impl Editable for Texture {
    fn create_editor<T: GroupExt>(&mut self, widget_parent: &mut T) -> Result<(), CreationFailure> {
        self.create_viewer(widget_parent)?;

        Ok(())

        // TODO: Make the editor here
    }
}
