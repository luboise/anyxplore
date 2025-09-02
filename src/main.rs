use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use bnl::{BNLFile, asset::AssetDescription};
use fltk::{
    app::{self},
    frame,
    group::Scroll,
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

    tree: tree::Tree,

    bnl_files: Vec<BNLStruct>,

    edit_window: Scroll,

    main_win: window::Window,
    frame: frame::Frame,
    count: i32,
    receiver: app::Receiver<Message>,
}

impl AnyXPloreApp {
    pub fn new() -> Self {
        let count = 0;
        let app = app::App::default();

        let (s, receiver) = app::channel();
        let mut main_win = window::Window::default()
            .with_size(400, 300)
            .with_label("AnyXPlore");

        main_win.make_resizable(true);

        let frame = frame::Frame::default().with_label(&count.to_string());
        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            eprintln!("Not enough args.");
            std::process::exit(1);
        }

        let mut tree = tree::Tree::default().with_size(300, 300);

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

        let scroll = fltk::group::Scroll::new(0, 0, 1000, 1000, "Edit Window");

        tree.emit(s, Message::TreeClicked);

        main_win.add(&tree);

        // let bruh = Scroll::default().with_size(1000, 1000).with_label("data");
        // main_win.add(&bruh);

        main_win.end();
        main_win.show();

        Self {
            app,
            tree,
            main_win,
            frame,
            edit_window: scroll,
            count,
            receiver,
            bnl_files,
        }
    }

    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::TreeClicked => {
                        if let Some(node) = self.tree.first_selected_item() {
                            let opt_node_data: Option<NodeData> = unsafe { node.user_data() };

                            match opt_node_data {
                                Some(node_data) => {
                                    if node_data.is_root {
                                        if let Some(bnl_file) =
                                            self.bnl_files.get_mut(node_data.bnl_index)
                                        {
                                            if bnl_file.descriptions.is_none() {
                                                println!(
                                                    "Loading asset descriptions for {}",
                                                    bnl_file.path.display()
                                                );

                                                let descriptions = Vec::new();

                                                bnl_file.data.asset_descriptions().iter().for_each(
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

                                                bnl_file.descriptions = Some(descriptions);
                                            }
                                        }
                                    }
                                }
                                None => todo!(),
                            }

                            /*
                            match data {

                                Some(data) => {
                                    self.bnl_files.values().find(|val|{val.tree_id == data.id})
                                    data.asset_descriptions().iter().for_each(|desc| {
                                        self.tree.insert(&node, desc.name(), i32::MAX);
                                    });
                                }
                                None => eprintln!("No data available."),
                            }
                            */
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
