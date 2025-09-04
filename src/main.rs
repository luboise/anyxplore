use std::{
    any::Any,
    env,
    ffi::OsStr,
    fmt::Display,
    fs::{self, DirEntry, ReadDir},
    io,
    path::{Path, PathBuf},
    time,
};

use bnl::{
    BNLFile,
    asset::{AssetDescription, texture::Texture},
    game::AssetType,
};
use eframe::egui::{
    self, Id,
    ahash::{HashMap, HashSet},
};
use egui_ltreeview::{Action, RowLayout, TreeView, TreeViewSettings};

use crate::editors::{Editable, Viewable};

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
    fn data(&self) -> Option<&BNLInners> {
        self.inners.as_ref()
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

                        if bnl_struct.data().is_none() {
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
            if let Some(selected_id) = self.selected_id {
                println!("SELECTED {:?}", selected_id);

                let asset_struct = self.asset_map.get(&selected_id).unwrap();
                let bnl_file = &self
                    .bnl_map
                    .get(&asset_struct.bnl_id)
                    .unwrap()
                    .data()
                    .unwrap()
                    .bnl_file;

                let raw_asset = bnl_file.get_raw_asset(&asset_struct.name).unwrap();

                match raw_asset.asset_type {
                    AssetType::ResTexture => {
                        let texture: Texture = bnl_file.get_asset(&asset_struct.name).unwrap();

                        texture.create_viewer(ui);
                    }

                    _ => (),
                }
            }
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

/*
    pub fn new() -> Self {
        let app = app::App::default();

        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            eprintln!("Not enough args.");
            std::process::exit(1);
        }

        let (s, receiver) = app::channel();
        let mut main_win = window::Window::default()
            .with_size(1024, 768)
            .with_label("AnyXPlore");

        let mut root = Flex::default().size_of_parent().with_type(FlexType::Row);
        main_win.add_resizable(&root);
        main_win.end();
        // main_win.make_resizable(true);

        let mut tree = tree::Tree::default();
        tree.set_show_root(false);
        tree.emit(s, Message::TreeClicked);

        let filename = Path::new(&args[1])
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let pb: PathBuf = args[1].clone().into();

        let bytes = fs::read(&args[1]).expect("Unable to read.");
        let new_bnl = BNLFile::from_bytes(&bytes).unwrap();

        let mut node = tree.add(&filename).unwrap();

        node.set_user_data(NodeData {
            is_root: true,
            bnl_index: 0,
        });

        let tree_id = 0;

        let bnl_files = vec![BNLStruct {
            tree_id,
            descriptions: None,
            bnl_file: new_bnl,
            path: pb,
        }];

        root.add(&tree);
        root.fixed(&tree, 350);
        let edit_window = EditWindow::new(&mut root).expect("Unable to create edit window.");

        root.show();
        root.end();

        main_win.show();
        main_win.end();

        Self {
            app,
            tree,
            main_win,
            edit_window,
            receiver,
            bnl_structs: bnl_files,
            flex: root,
        }
    }

    pub fn on_tree_item_clicked(&mut self) -> Result<(), XError> {
        let node = self.tree.first_selected_item().ok_or(XError::TreeError)?;
        let node_data: NodeData = unsafe { node.user_data() }.ok_or_else(|| {
            eprintln!("No node data available.");
            XError::TreeError
        })?;

        // If a root node is clicked, ie a BNL file, handle it
        if node_data.is_root {
            let bnl_struct = self
                .bnl_structs
                .get_mut(node_data.bnl_index)
                .ok_or(XError::TreeError)?;

            // If the descriptions aren't loaded, load them and put them into the tree
            if bnl_struct.descriptions.is_none() {
                bnl_struct
                    .load_asset_descriptions()?
                    .iter()
                    .for_each(|desc| {
                        let mut new_node = self.tree.insert(&node, desc.name(), i32::MAX).unwrap();

                        new_node.set_user_data(NodeData {
                            is_root: false,
                            bnl_index: node_data.bnl_index,
                        });
                    });
            }

            Ok(())
        } else {
            let bnl_struct = self
                .bnl_structs
                .get_mut(node_data.bnl_index)
                .ok_or(XError::NodeError)?;

            let asset_name = node.label().unwrap_or_default();

            match bnl_struct
                .bnl_file
                .get_raw_asset(&asset_name)
                .map_err(|_| XError::NodeError)?
                .asset_type
            {
                AssetType::ResTexture => {
                    if let Ok(tex) = bnl_struct.bnl_file.get_asset::<Texture>(&asset_name) {
                        self.edit_window.reset_begin();
                        self.edit_window.add_viewer(tex);
                        self.edit_window.reset_end();
                    }
                }
                t => {
                    println!("Type {:?} not implemented yet.", t);
                }
            };

            Ok(())
        }
    }

    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::TreeClicked => self.on_tree_item_clicked().unwrap_or_else(|e| {
                        eprintln!("Unable to handle tree click.\nError: {}", e);
                    }),
                }
            }
        }
    }
}


// fn main() {
// let app = AnyXPloreApp::new();
// app.run();
// }


*/

fn main() {
    let args: Vec<String> = env::args().collect();

    let native_options = eframe::NativeOptions::default();
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
