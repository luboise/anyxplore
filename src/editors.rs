use std::{
    io::{Cursor, Read},
    time::SystemTime,
};

use bnl::asset::{
    Asset,
    model::Model,
    script::{Script, ScriptDescriptor, ScriptParamType},
    texture::{Texture, TextureDescriptor},
};
use byteorder::{LittleEndian, ReadBytesExt};
use eframe::egui::{
    self, ColorImage, Id, ImageSource, TextureHandle, TextureId, Vec2, load::SizedTexture,
};

use crate::Message;

#[derive(Debug)]
pub enum CreationFailure {
    PartialFailure(String),
    CompleteFailure(String),
}

pub struct ViewerContext<'a> {
    ui: &'a mut egui::Ui,
    viewer_index: usize,
}

impl<'a> ViewerContext<'a> {
    pub fn new(ui: &'a mut egui::Ui) -> ViewerContext<'a> {
        ViewerContext {
            ui,
            viewer_index: 0,
        }
    }

    pub fn next_viewer_index(&mut self) -> usize {
        let index = self.viewer_index;
        self.viewer_index += 1;

        index
    }

    pub fn ui_mut(&mut self) -> &mut &'a mut egui::Ui {
        &mut self.ui
    }
}

pub trait Viewable {
    fn create_viewer(&self, ctx: &mut ViewerContext) -> Result<(), CreationFailure>;
}

pub trait Editable: Viewable {
    fn create_editor(&mut self, ctx: &mut ViewerContext) -> Result<(), CreationFailure>;
}

impl Viewable for TextureDescriptor {
    fn create_viewer(&self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        ctx.ui.heading("Texture");

        let index = ctx.next_viewer_index();

        egui::Grid::new(format!("texture_{}", index)).show(ctx.ui, |ui| {
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
    fn create_viewer(&self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        self.descriptor().create_viewer(ctx)?;

        if let Ok(rgba) = self.to_rgba_image() {
            let color_image =
                ColorImage::from_rgba_unmultiplied([rgba.width(), rgba.height()], rgba.bytes());

            let texture: TextureHandle = ctx.ui.ctx().load_texture(
                "some texture",
                color_image,
                egui::TextureOptions::LINEAR,
            );

            ctx.ui.image(&texture);
        } else {
            ctx.ui.label("Error creating image view.");
        }

        Ok(())
    }
}

impl Editable for Texture {
    fn create_editor(&mut self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        self.descriptor().create_viewer(ctx)?;

        // TODO: Make the editor here

        Ok(())
    }
}

impl Viewable for Model {
    fn create_viewer(&self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        let textures = self.textures().ok_or(CreationFailure::CompleteFailure(
            "No textures available for model.".to_string(),
        ))?;

        for texture in textures {
            texture.create_viewer(ctx)?;
        }

        Ok(())
    }
}

impl Viewable for ScriptDescriptor {
    fn create_viewer(&self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        ctx.ui
            .heading(format!("Script ({} Operations)", self.operations().len()));

        egui::Grid::new("script_viewer").show(ctx.ui, |ui| {
            self.operations().iter().for_each(|op| {
                ui.label(format!("{:?}", op.opcode()));

                let shape = op.get_shape();

                let operand_bytes = op.operand_bytes();

                let mut cur = Cursor::new(operand_bytes);

                for (key, value) in shape {
                    // TODO: Add other cases later
                    match value.param_type() {
                        ScriptParamType::F32 => {
                            let val = cur.read_f32::<LittleEndian>().unwrap_or(0.0);
                            ui.label(format!("{:.1}", val)).on_hover_text(key);
                        }
                        ScriptParamType::F64 => {
                            let val = cur.read_f64::<LittleEndian>().unwrap_or(0.0);
                            ui.label(format!("{:.2}", val)).on_hover_text(key);
                        }
                        ScriptParamType::U8 => {
                            let val = cur.read_u8().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::I8 => {
                            let val = cur.read_i8().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::I16 => {
                            let val = cur.read_i16::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::U16 => {
                            let val = cur.read_u16::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::I32 => {
                            let val = cur.read_i32::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::U32 => {
                            let val = cur.read_u32::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::I64 => {
                            let val = cur.read_i64::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::U64 => {
                            let val = cur.read_u64::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }

                        ScriptParamType::Bytes(count) => {
                            let mut v = vec![0x00; *count];
                            cur.read_exact(&mut v).unwrap_or_default();

                            let hex: String = v.iter().map(|b| format!("{:02x}", b)).collect();
                            ui.label(hex).on_hover_text(key);
                        }
                        ScriptParamType::String(size) => {
                            let mut v = vec![0x00; *size];
                            cur.read_exact(&mut v).unwrap_or_default();

                            let hex: String = v
                                .iter()
                                .take(0x80)
                                .take_while(|&&b| b != 0x00)
                                .map(|&b| b as char)
                                .collect();

                            ui.label(hex).on_hover_text(key);
                        }

                        _ => (),
                    }
                }

                ui.end_row();
            });
        });

        Ok(())
    }
}

impl Editable for ScriptDescriptor {
    fn create_editor(&mut self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        ctx.ui
            .heading(format!("Script ({} Operations)", self.operations().len()));

        let mut deleted_index = None;

        egui::Grid::new("script_viewer").show(ctx.ui, |ui| {
            self.operations().iter().enumerate().for_each(|(i, op)| {
                if ui.button("x").clicked() {
                    deleted_index = Some(i);
                }

                ui.label(format!("{:?}", op.opcode()));

                let shape = op.get_shape();

                let operand_bytes = op.operand_bytes();

                let mut cur = Cursor::new(operand_bytes);

                for (key, value) in shape {
                    // TODO: Add other cases later
                    match value.param_type() {
                        ScriptParamType::F32 => {
                            let val = cur.read_f32::<LittleEndian>().unwrap_or(0.0);
                            ui.label(format!("{:.1}", val)).on_hover_text(key);
                        }
                        ScriptParamType::F64 => {
                            let val = cur.read_f64::<LittleEndian>().unwrap_or(0.0);
                            ui.label(format!("{:.2}", val)).on_hover_text(key);
                        }
                        ScriptParamType::U8 => {
                            let val = cur.read_u8().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::I8 => {
                            let val = cur.read_i8().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::I16 => {
                            let val = cur.read_i16::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::U16 => {
                            let val = cur.read_u16::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::I32 => {
                            let val = cur.read_i32::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::U32 => {
                            let val = cur.read_u32::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::I64 => {
                            let val = cur.read_i64::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ScriptParamType::U64 => {
                            let val = cur.read_u64::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }

                        ScriptParamType::Bytes(count) => {
                            let mut v = vec![0x00; *count];
                            cur.read_exact(&mut v).unwrap_or_default();

                            let hex: String = v.iter().map(|b| format!("{:02x}", b)).collect();
                            ui.label(hex).on_hover_text(key);
                        }
                        ScriptParamType::String(size) => {
                            let mut v = vec![0x00; *size];
                            cur.read_exact(&mut v).unwrap_or_default();

                            let hex: String = v
                                .iter()
                                .take(0x80)
                                .take_while(|&&b| b != 0x00)
                                .map(|&b| b as char)
                                .collect();

                            ui.label(hex).on_hover_text(key);
                        }

                        _ => (),
                    }
                }

                ui.end_row();
            });
        });

        if let Some(index) = deleted_index {
            println!("Call to delete index {}", index);
        }

        Ok(())
    }
}

impl Viewable for Script {
    fn create_viewer(&self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        self.descriptor().create_viewer(ctx)?;

        Ok(())
    }
}
