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
use eframe::egui::{self, Id};
use egui_ltreeview::{RowLayout, TreeView, TreeViewSettings};
use fltk::{
    app::{self},
    button, frame,
    group::{Flex, FlexType, Pack, PackType, Scroll, ScrollType},
    image::RgbImage,
    prelude::*,
    tree::{self, TreeItem},
    window,
};
use walkdir::WalkDir;

use crate::editors::{Editable, Viewable};

// mod edit_window;
mod editors;
mod widgets;

#[derive(Copy, Clone)]
enum Message {
    TreeClicked,
}

struct BNLStruct {
    tree_id: u32,

    path: PathBuf,

    descriptions: Option<Vec<AssetDescription>>,
    bnl_file: BNLFile,
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

impl BNLStruct {
    pub fn load_asset_descriptions(&mut self) -> Result<&Vec<AssetDescription>, XError> {
        println!("Loading asset descriptions for {}", self.path.display());

        let descriptions = self.bnl_file.asset_descriptions().to_vec();
        self.descriptions = Some(descriptions);

        // Unwrap can't fail since we just assigned it
        Ok(&self.descriptions.as_ref().unwrap())
    }
}

#[derive(Clone)]
struct NodeData {
    is_root: bool,
    bnl_index: usize,
}

#[derive(Default)]
struct AnyXPloreApp {
    app: app::App,

    directory: PathBuf,
    directory_valid: bool,

    // flex: fltk::group::Flex,
    // tree: tree::Tree,
    bnl_structs: Vec<BNLStruct>,
    // edit_window: EditWindow,
    // main_win: window::Window,
    // receiver: app::Receiver<Message>,
}

fn create_file_tree(
    path: &PathBuf,
    builder: &mut egui_ltreeview::TreeViewBuilder<'_, Id>,
) -> Result<(), io::Error> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let path = entry?.path().clone();

        if path.is_file() {
            builder.leaf(
                Id::new(&path),
                path.file_name()
                    .expect("bruh")
                    .to_str()
                    .map(|val| val.to_string())
                    .unwrap_or("errorfile".to_string()),
            );
        } else if path.is_dir() {
            builder.dir(
                Id::new(&path),
                path.file_name()
                    .expect("bruh")
                    .to_str()
                    .map(|val| val.to_string())
                    .unwrap_or("errorfile".to_string()),
            );
            create_file_tree(&path, builder)?;
            builder.close_dir();
        }
    }

    Ok(())
}

impl eframe::App for AnyXPloreApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            ui.heading("Hello World!");

            egui::ScrollArea::vertical().show(ui, |ui| {
                let (a, b) = TreeView::new(Id::new("tree view"))
                    .with_settings(TreeViewSettings {
                        row_layout: RowLayout::CompactAlignedLabels,
                        ..Default::default()
                    })
                    .show(ui, |builder| {
                        create_file_tree(&self.directory, builder)
                            .unwrap_or_else(|_| eprintln!("Error while building tree."));
                    });
                for action in b {
                    match action {
                        egui_ltreeview::Action::Activate(activated) => {
                            println!("Activated {:?}", activated.selected);
                        }
                        _ => (),
                    }
                }
            });
        });
    }
}

impl AnyXPloreApp {
    fn new(cc: &eframe::CreationContext<'_>, dir: Option<PathBuf>) -> Self {
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
