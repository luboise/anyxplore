use std::{
    env, fs,
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
    tree::{self},
    window,
};

mod widgets;

#[derive(Copy, Clone)]
enum Message {
    TreeClicked,
}

struct BNLStruct {
    tree_id: u32,

    path: PathBuf,

    descriptions: Option<Vec<AssetDescription>>,
    data: BNLFile,
}

#[derive(Clone)]
struct NodeData {
    is_root: bool,
    bnl_index: usize,
}

struct AnyXPloreApp {
    app: app::App,

    flex: fltk::group::Flex,

    image_frame: fltk::frame::Frame,

    tree: tree::Tree,

    bnl_files: Vec<BNLStruct>,

    edit_window: Scroll,

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
            data: new_bnl,
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
            edit_window: scroll,
            count,
            receiver,
            bnl_files,
            flex,
            image_frame,
        }
    }

    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::TreeClicked => {
                        if let Some(node) = self.tree.first_selected_item() {
                            let node_data: NodeData = match unsafe { node.user_data() } {
                                Some(d) => d,
                                None => {
                                    eprintln!("No node data available.");
                                    return;
                                }
                            };

                            if node_data.is_root {
                                if let Some(bnl_struct) =
                                    self.bnl_files.get_mut(node_data.bnl_index)
                                {
                                    if bnl_struct.descriptions.is_none() {
                                        println!(
                                            "Loading asset descriptions for {}",
                                            bnl_struct.path.display()
                                        );

                                        let descriptions = Vec::new();

                                        bnl_struct.data.asset_descriptions().iter().for_each(
                                            |desc| {
                                                let mut new_node = self
                                                    .tree
                                                    .insert(&node, desc.name(), i32::MAX)
                                                    .unwrap();

                                                new_node.set_user_data(NodeData {
                                                    is_root: false,
                                                    bnl_index: node_data.bnl_index,
                                                });
                                            },
                                        );

                                        bnl_struct.descriptions = Some(descriptions);
                                    }
                                }
                            } else if let Some(bnl_struct) =
                                self.bnl_files.get_mut(node_data.bnl_index)
                            {
                                let asset_name = node.label().unwrap_or_default();

                                if let Ok(raw) = bnl_struct.data.get_raw_asset(&asset_name) {
                                    if raw.asset_type == AssetType::ResTexture {
                                        if let Ok(tex) =
                                            bnl_struct.data.get_asset::<Texture>(&asset_name)
                                        {
                                            if let Ok(rgba) = tex.to_rgba_image() {
                                                if let Ok(img) = unsafe {
                                                    RgbImage::from_data(
                                                        rgba.bytes(),
                                                        rgba.width() as i32,
                                                        rgba.height() as i32,
                                                        fltk::enums::ColorDepth::Rgba8,
                                                    )
                                                } {
                                                    println!("Setting image.");

                                                    self.image_frame.resize(
                                                        self.image_frame.x(),
                                                        self.image_frame.y(),
                                                        img.width(),
                                                        img.height(),
                                                    );

                                                    // self.image_frame.set_image::<RgbImage>(None);
                                                    // self.image_frame.redraw();

                                                    self.image_frame.set_image(Some(img));

                                                    self.image_frame.parent().unwrap().redraw();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let app = AnyXPloreApp::new();
    app.run();
}
