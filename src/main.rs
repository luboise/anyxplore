use std::{
    env,
    fmt::Display,
    fs::{self},
    io,
    path::PathBuf,
};

use bnl::{
    BNLFile,
    asset::{Asset, AssetDescription, model::Model, script::Script, texture::Texture},
    game::AssetType,
};
use eframe::egui::{
    self, Id,
    ahash::{HashMap, HashSet},
};
use egui_file_dialog::FileDialog;
use egui_ltreeview::{Action, RowLayout, TreeView, TreeViewSettings};

use crate::editors::{Editable, Viewable, ViewerContext};

use image::ImageReader;

// mod edit_window;
mod editors;
mod widgets;

#[derive(Copy, Clone)]
enum Message {
    TreeClicked,
}

#[derive(Debug)]
struct BNLInners {
    bnl_file: BNLFile,
    descriptions: Vec<AssetDescription>,
}

impl BNLInners {
    fn bnl_file(&self) -> &BNLFile {
        &self.bnl_file
    }

    fn bnl_file_mut(&mut self) -> &mut BNLFile {
        &mut self.bnl_file
    }

    // println!("Loading asset descriptions for {}", self.path.display());

    pub fn load_asset_descriptions(&mut self) -> Result<&Vec<AssetDescription>, XError> {
        self.descriptions = self.bnl_file.asset_descriptions().to_vec();
        Ok(&self.descriptions)
    }

    pub fn from_bnl_bytes(bytes: &[u8]) -> Result<Self, std::io::Error> {
        let bnl_file = bnl::BNLFile::from_bytes(&bytes).expect("Unable to create BNL file.");
        let descriptions = bnl_file.asset_descriptions().to_owned();

        Ok(BNLInners {
            bnl_file,
            descriptions,
        })
    }
}

#[derive(Debug, Default)]
struct BNLStruct {
    path: PathBuf,
    inners: Option<BNLInners>,
}

impl BNLStruct {
    fn inners(&self) -> Option<&BNLInners> {
        self.inners.as_ref()
    }

    fn inners_mut(&mut self) -> &mut Option<BNLInners> {
        &mut self.inners
    }
}

#[derive(Debug)]
enum XError {
    AlreadyLoaded,
    TreeError,
    NodeError,
}

impl Display for XError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
struct NodeData {
    is_root: bool,
    bnl_index: usize,
}

#[derive(Clone)]
struct AssetStruct {
    name: String,
    bnl_id: Id,
}

#[derive(Default)]
struct AnyXPloreApp {
    asset_map: HashMap<Id, AssetStruct>, // Maps an aid to its parent BNL file
    bnl_map: HashMap<Id, BNLStruct>,     // Maps a BNL id to its BNL struct

    selected_id: Option<Id>,

    directory: PathBuf,
    directory_valid: bool,

    // flex: fltk::group::Flex,
    // tree: tree::Tree,
    bnl_structs: Vec<BNLStruct>,
    // edit_window: EditWindow,
    // main_win: window::Window,
    // receiver: app::Receiver<Message>,
    dropped_files: Vec<egui::DroppedFile>,

    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,
}

impl AnyXPloreApp {
    fn create_file_tree(
        &mut self,
        path: &PathBuf,
        builder: &mut egui_ltreeview::TreeViewBuilder<'_, Id>,
        // paths: &mut HashMap<Id, PathBuf>,
    ) -> Result<(), io::Error> {
        let entries = fs::read_dir(path)?;

        for entry in entries {
            let path = entry?.path().clone();

            let bnl_id = Id::new(&path);

            // If we find a bnl file
            if path.is_file() && path.extension().unwrap_or_default() == "bnl" {
                // Create a BNLStruct if one doesn't already exist
                if !self.bnl_map.contains_key(&bnl_id) {
                    self.bnl_map.insert(
                        bnl_id,
                        BNLStruct {
                            path: path.clone(),
                            ..Default::default()
                        },
                    );
                }

                // We can unwrap because we just made sure its available
                let bnl_struct = self.bnl_map.get_mut(&bnl_id).unwrap();

                if let Some(inners) = &bnl_struct.inners {
                    builder.dir(
                        bnl_id,
                        path.file_name()
                            .expect("bruh")
                            .to_str()
                            .map(|val| val.to_string())
                            .unwrap_or("errorfile".to_string()),
                    );

                    inners.descriptions.iter().for_each(|desc| {
                        let name = desc.name();

                        let aid_id = Id::new(path.join(name)); // Id generated from full path + aid

                        self.asset_map.insert(
                            aid_id,
                            AssetStruct {
                                name: name.to_string(),
                                bnl_id,
                            },
                        );
                        builder.leaf(aid_id, name);
                    });

                    builder.close_dir();
                } else {
                    builder.leaf(
                        bnl_id,
                        path.file_name()
                            .expect("bruh")
                            .to_str()
                            .map(|val| val.to_string())
                            .unwrap_or("errorfile".to_string()),
                    );
                }
            } else if path.is_dir() {
                builder.dir(
                    Id::new(&path),
                    path.file_name()
                        .expect("bruh")
                        .to_str()
                        .map(|val| val.to_string())
                        .unwrap_or("errorfile".to_string()),
                );
                self.create_file_tree(&path, builder)?;
                builder.close_dir();
            }
        }

        Ok(())
    }
}

impl eframe::App for AnyXPloreApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.dropped_files.clear();
        self.dropped_files
            .extend(ctx.input(|i| i.raw.dropped_files.clone()));

        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            ui.heading("AnyXplore");

            egui::ScrollArea::vertical().show(ui, |ui| {
                let tree = TreeView::new(Id::new("tree view")).with_settings(TreeViewSettings {
                    row_layout: RowLayout::CompactAlignedLabels,
                    ..Default::default()
                });

                let (_response, actions) = tree.show(ui, |builder| {
                    self.create_file_tree(&self.directory.clone(), builder)
                        .unwrap_or_else(|_| eprintln!("Error while building tree."));
                });
                for action in actions {
                    if let Action::Activate(activated) = action {
                        let id = activated.selected[0];

                        // If the thing clicked was an asset, and it has a bnl file
                        if let Some(asset_mapping) = self.asset_map.get(&id) {
                            if self.bnl_map.contains_key(&asset_mapping.bnl_id) {
                                self.selected_id = Some(id);
                            } else {
                                eprintln!("Asset mapping exists but no BNL exists for the asset.");
                            }
                        }

                        if !self.bnl_map.contains_key(&id) {
                            continue;
                        }

                        let bnl_struct = self.bnl_map.get_mut(&id).unwrap();

                        if bnl_struct.inners().is_none() {
                            let bnl_inners = BNLInners::from_bnl_bytes(
                                &fs::read(&bnl_struct.path).unwrap_or_default(),
                            );

                            match bnl_inners {
                                Ok(inners) => bnl_struct.inners = Some(inners),
                                Err(e) => eprintln!("Unable to load BNL file.\nError: {}", e),
                            }
                        }
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(selected_id) = self.selected_id {
                    let asset_struct = self.asset_map.get(&selected_id).unwrap();

                    let bnl_struct = self
                        .bnl_map
                        .get_mut(&asset_struct.bnl_id)
                        .expect("Unable to get BNL struct.");

                    let bnl_path = bnl_struct.path.clone();

                    if let Some(inners) = bnl_struct.inners_mut() {
                        let bnl_file: &mut BNLFile = &mut inners.bnl_file;
                        // Now you can mutate `bnl_file` as needed

                        let raw_asset = bnl_file.get_raw_asset(&asset_struct.name).unwrap();

                        let mut viewer_ctx = ViewerContext::new(ui);

                        match raw_asset.asset_type {
                            AssetType::ResTexture => {
                                let mut texture: Texture =
                                    bnl_file.get_asset(&asset_struct.name).unwrap();
                                texture.create_viewer(&mut viewer_ctx);

                                if ui.button("Set Texture").clicked() {
                                    self.file_dialog.pick_file();
                                }

                                self.file_dialog.update(ctx);

                                if let Some(path) = self.file_dialog.take_picked() {
                                    self.picked_file = Some(path.to_path_buf());
                                }

                                if let Some(file) = self.picked_file.take() {
                                    let img = ImageReader::open(&file)
                                        .expect("Unable to open image")
                                        .decode()
                                        .expect("Unable to decode image");

                                    texture.set_from_rgba(
                                        texture.descriptor().width() as usize,
                                        texture.descriptor().height() as usize,
                                        img.as_bytes(),
                                    );

                                    println!("Updating image");
                                    bnl_file.update_asset(&asset_struct.name, &texture);

                                    fs::write(bnl_path, &bnl_file.to_bytes())
                                        .expect("Unable to write");
                                }
                            }
                            AssetType::ResModel => {
                                let model: Model = bnl_file.get_asset(&asset_struct.name).unwrap();

                                model.create_viewer(&mut viewer_ctx);

                                /*
                                if ui.button("Set Texture").clicked() {
                                    self.file_dialog.pick_file();
                                }

                                self.file_dialog.update(ctx);

                                if let Some(path) = self.file_dialog.take_picked() {
                                    self.picked_file = Some(path.to_path_buf());
                                }

                                if let Some(file) = self.picked_file.take() {
                                    let img = ImageReader::open(&file)
                                        .expect("Unable to open image")
                                        .decode()
                                        .expect("Unable to decode image");

                                    texture.set_from_rgba(
                                        texture.descriptor().width() as usize,
                                        texture.descriptor().height() as usize,
                                        img.as_bytes(),
                                    );

                                    println!("Updating image");
                                    bnl_file.update_asset(&asset_struct.name, &texture);

                                    fs::write(bnl_path, &bnl_file.to_bytes()).expect("Unable to write");
                                }
                                */
                            }
                            AssetType::ResScript => {
                                if let Ok(script) = bnl_file.get_asset::<Script>(&asset_struct.name)
                                {
                                    script.create_viewer(&mut viewer_ctx);
                                } else {
                                    viewer_ctx.ui_mut().heading("Error parsing script.");
                                }
                            }

                            _ => (),
                        }
                    }
                }
            });
        });
    }
}

impl AnyXPloreApp {
    fn new(_cc: &eframe::CreationContext<'_>, dir: Option<PathBuf>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        match dir {
            Some(d) => AnyXPloreApp {
                directory: d,
                directory_valid: false,
                ..Default::default()
            },

            None => AnyXPloreApp {
                directory_valid: false,
                ..Default::default()
            },
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let native_options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(AnyXPloreApp::new(
                cc,
                args.get(1).map(PathBuf::from),
            )))
        }),
    );
}
