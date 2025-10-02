use std::io::{Cursor, Read};

use bnl::{
    asset::{
        Asset,
        model::Model,
        param::{HasParams, ParamType},
        script::{Script, ScriptDescriptor},
        texture::{Texture, TextureData, TextureDescriptor},
    },
    game::AssetType,
};
use byteorder::{LittleEndian, ReadBytesExt};
use eframe::egui::{self, ColorImage, TextureHandle};

use crate::Message;

#[derive(Debug)]
pub enum CreationFailure {
    PartialFailure(String),
    CompleteFailure(String),
}

#[derive(Debug)]
pub struct DeletionRequest {
    pub asset_type: AssetType,
    pub deletion_index: usize,
}

pub struct ViewerContext<'a> {
    ui: &'a mut egui::Ui,
    viewer_index: usize,

    pub(crate) update_bnl: bool,

    delete_request: Option<DeletionRequest>,
}

impl<'a> ViewerContext<'a> {
    pub fn new(ui: &'a mut egui::Ui) -> ViewerContext<'a> {
        ViewerContext {
            ui,
            viewer_index: 0,
            delete_request: None,
            update_bnl: false,
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

    pub fn delete_request_mut(&mut self) -> &mut Option<DeletionRequest> {
        &mut self.delete_request
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

impl Viewable for TextureData {
    fn create_viewer(&self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        let descriptor = self.descriptor();

        descriptor.create_viewer(ctx)?;

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

impl Viewable for Texture {
    fn create_viewer(&self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        self.data().create_viewer(ctx)
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
                        ParamType::F32 => {
                            let val = cur.read_f32::<LittleEndian>().unwrap_or(0.0);
                            ui.label(format!("{:.1}", val)).on_hover_text(key);
                        }
                        ParamType::F64 => {
                            let val = cur.read_f64::<LittleEndian>().unwrap_or(0.0);
                            ui.label(format!("{:.2}", val)).on_hover_text(key);
                        }
                        ParamType::U8 => {
                            let val = cur.read_u8().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ParamType::I8 => {
                            let val = cur.read_i8().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ParamType::I16 => {
                            let val = cur.read_i16::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ParamType::U16 => {
                            let val = cur.read_u16::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ParamType::I32 => {
                            let val = cur.read_i32::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ParamType::U32 => {
                            let val = cur.read_u32::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ParamType::I64 => {
                            let val = cur.read_i64::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }
                        ParamType::U64 => {
                            let val = cur.read_u64::<LittleEndian>().unwrap_or(0);
                            ui.label(format!("{}", val)).on_hover_text(key);
                        }

                        ParamType::Bytes(count) => {
                            let mut v = vec![0x00; *count];
                            cur.read_exact(&mut v).unwrap_or_default();

                            let hex: String = v.iter().map(|b| format!("{:02x}", b)).collect();
                            ui.label(hex).on_hover_text(key);
                        }
                        ParamType::String(size) => {
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

impl Editable for Script {
    fn create_editor(&mut self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        self.descriptor_mut().create_editor(ctx)
    }
}

impl Editable for ScriptDescriptor {
    fn create_editor(&mut self, ctx: &mut ViewerContext) -> Result<(), CreationFailure> {
        ctx.ui
            .heading(format!("Script ({} Operations)", self.operations().len()));

        let mut deletion_index = None;

        egui::Grid::new("script_viewer").show(ctx.ui, |ui| {
            self.operations_mut()
                .iter_mut()
                .enumerate()
                .for_each(|(i, op)| {
                    if ui.button("x").clicked() {
                        if deletion_index.is_none() {
                            deletion_index = Some(i);
                        }
                    }

                    ui.label(format!("{:?}", op.opcode()));

                    let shape = op.get_shape();

                    let operand_bytes = op.operand_bytes().to_vec();

                    let mut cur = Cursor::new(operand_bytes);

                    for (key, value) in shape {
                        // TODO: Add other cases later
                        match value.param_type() {
                            ParamType::F32 => {
                                let val = cur.read_f32::<LittleEndian>().unwrap_or(0.0);

                                let mut text = format!("{:.1}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<f32>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }
                            ParamType::F64 => {
                                let val = cur.read_f64::<LittleEndian>().unwrap_or(0.0);

                                let mut text = format!("{:.2}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<f64>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }
                            ParamType::U8 => {
                                let val = cur.read_u8().unwrap_or(0);

                                let mut text = format!("{}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<u8>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }
                            ParamType::I8 => {
                                let val = cur.read_i8().unwrap_or(0);

                                let mut text = format!("{}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<i8>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }
                            ParamType::U16 => {
                                let val = cur.read_u16::<LittleEndian>().unwrap_or(0);

                                let mut text = format!("{}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<u16>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }
                            ParamType::I16 => {
                                let val = cur.read_i16::<LittleEndian>().unwrap_or(0);

                                let mut text = format!("{}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<i16>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }

                            ParamType::U32 => {
                                let val = cur.read_u32::<LittleEndian>().unwrap_or(0);

                                let mut text = format!("{}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<u32>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }
                            ParamType::I32 => {
                                let val = cur.read_i32::<LittleEndian>().unwrap_or(0);

                                let mut text = format!("{}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<i32>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }

                            ParamType::U64 => {
                                let val = cur.read_u64::<LittleEndian>().unwrap_or(0);

                                let mut text = format!("{}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<u64>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }
                            ParamType::I64 => {
                                let val = cur.read_i64::<LittleEndian>().unwrap_or(0);

                                let mut text = format!("{}", val);
                                if ui.text_edit_singleline(&mut text).changed() {
                                    if let Ok(val) = text.parse::<i64>() {
                                        op.set_param_by_name(&key, val).map_err(|e| {
                                            CreationFailure::PartialFailure(format!("{:?}", e))
                                        });

                                        ctx.update_bnl = true;
                                    }
                                }
                            }

                            ParamType::Bytes(count) => {
                                let mut v = vec![0x00; *count];
                                cur.read_exact(&mut v).unwrap_or_default();

                                let hex: String = v.iter().map(|b| format!("{:02x}", b)).collect();
                                ui.label(hex).on_hover_text(key);
                            }
                            ParamType::String(size) => {
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

        if ctx.delete_request.is_none() {
            if let Some(index) = deletion_index {
                {
                    ctx.delete_request = Some(DeletionRequest {
                        asset_type: AssetType::ResScript,
                        deletion_index: index,
                    });
                }
            }
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
