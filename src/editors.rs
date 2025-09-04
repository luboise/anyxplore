use bnl::asset::{
    Asset,
    texture::{Texture, TextureDescriptor},
};
use eframe::egui::{self, ImageSource, TextureId, load::SizedTexture};

use crate::Message;

#[derive(Debug)]
pub enum CreationFailure {
    PartialFailure(String),
    CompleteFailure(String),
}

pub trait Viewable {
    fn create_viewer(&self, ui: &mut egui::Ui) -> Result<(), CreationFailure>;
}

pub trait Editable: Viewable {
    fn create_editor(&mut self, ui: &mut egui::Ui) -> Result<(), CreationFailure>;
}

impl Viewable for TextureDescriptor {
    fn create_viewer(&self, ui: &mut egui::Ui) -> Result<(), CreationFailure> {
        ui.heading("Texture");

        egui::Grid::new("some_unique_id").show(ui, |ui| {
            ui.label("Format");
            ui.label(format!("{:?}", self.format()));
            ui.end_row();

            ui.label("Header Size");
            ui.label(format!("{}", self.header_size()));
            ui.end_row();

            ui.label("Width");
            ui.label(format!("{}", self.width()));
            ui.end_row();

            ui.label("Height");
            ui.label(format!("{}", self.height()));
            ui.end_row();

            ui.label("Flags");
            ui.label(format!("{}", self.flags()));
            ui.end_row();

            ui.label("Unknown3a");
            ui.label(format!("{}", self.unknown_3a()));
            ui.end_row();

            ui.label("Resource Offset");
            ui.label(format!("{}", self.texture_offset()));
            ui.end_row();

            ui.label("Resource Size");
            ui.label(format!("{}", self.texture_size()));
            ui.end_row();
        });

        Ok(())
    }
}

impl Viewable for Texture {
    fn create_viewer(&self, ui: &mut egui::Ui) -> Result<(), CreationFailure> {
        self.descriptor().create_viewer(ui)?;

        if let Ok(rgba) = self.to_rgba_image() {
            // let image_source = ImageSource::Texture(SizedTexture::new(TextureId::User(()), size));

            // println!("Setting image.");

            // img.scale(frame.width(), frame.height(), true, true);

            // ui.image(image_source);
        } else {
            ui.label("Error creating image view.");
        }

        Ok(())
    }
}

impl Editable for Texture {
    fn create_editor(&mut self, ui: &mut egui::Ui) -> Result<(), CreationFailure> {
        self.descriptor().create_viewer(ui)?;

        // TODO: Make the editor here

        Ok(())
    }
}
