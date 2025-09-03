use std::{
    env,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use bnl::{
    BNLFile,
    asset::{AssetDescription, texture::Texture},
    game::AssetType,
};
use fltk::{
    app::{self},
    frame,
    group::{Flex, Scroll},
    image::RgbImage,
    prelude::*,
    tree::{self, TreeItem},
    window,
};

use crate::editors::{Editable, Viewable};

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

struct EditWindow {
    scroll: Scroll,
}

impl EditWindow {
    pub fn reset(&mut self) {
        self.scroll.clear()
    }

    pub fn add_viewer<V: Viewable>(&mut self, viewable: V) {
        viewable.create_viewer(&mut self.scroll);
        self.scroll.scroll_to(0, 0);
        self.scroll.redraw();
    }

    pub fn add_editor<E: Editable>(&mut self, editable: &mut E) {
        editable.create_editor(&mut self.scroll);

        self.scroll.scroll_to(0, 0);
        self.scroll.redraw();
    }

    fn scroll_mut(&mut self) -> &mut Scroll {
        &mut self.scroll
    }

    fn scroll(&self) -> &Scroll {
        &self.scroll
    }
}

struct AnyXPloreApp {
    app: app::App,

    flex: fltk::group::Flex,

    image_frame: fltk::frame::Frame,

    tree: tree::Tree,

    bnl_structs: Vec<BNLStruct>,

    edit_window: EditWindow,

    main_win: window::Window,

    count: i32,

    receiver: app::Receiver<Message>,
}

impl AnyXPloreApp {
    pub fn new() -> Self {
        let count = 0;
        let app = app::App::default();

        let (s, receiver) = app::channel();
        let mut main_win = window::Window::default()
            .with_size(1024, 768)
            .with_label("AnyXPlore");

        main_win.make_resizable(true);

        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            eprintln!("Not enough args.");
            std::process::exit(1);
        }

        let mut tree = tree::Tree::default();

        tree.set_show_root(false);

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

        tree.emit(s, Message::TreeClicked);

        let mut scroll = fltk::group::Scroll::default().with_label("Edit Window");

        scroll.show();

        let image_frame = fltk::frame::Frame::default();

        scroll.add(&image_frame);
        scroll.end();

        let mut flex = Flex::default().size_of_parent();
        flex.set_type(fltk::group::FlexType::Row);
        flex.add(&tree);
        flex.add(&scroll);

        main_win.add(&flex);

        main_win.end();
        main_win.show();

        Self {
            app,
            tree,
            main_win,
            edit_window: EditWindow { scroll },
            count,
            receiver,
            bnl_structs: bnl_files,
            flex,
            image_frame,
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

            return Ok(());
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
                        self.edit_window.reset();
                        self.edit_window.add_viewer(tex);
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

fn main() {
    let app = AnyXPloreApp::new();
    app.run();
}
