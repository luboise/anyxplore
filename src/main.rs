use std::{env, fs, path::Path, sync::Arc};

use bnl::BNLFile;
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

struct AnyXPloreApp {
    app: app::App,

    tree: tree::Tree,

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

        let bytes = fs::read(&args[1]).expect("Unable to read.");

        let mut tree = tree::Tree::default().with_size(300, 300);

        tree.set_show_root(false);

        let filename = Path::new(&args[1])
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut main_bnl = tree.add(&filename).unwrap();

        let bnl_arc = Arc::new(BNLFile::from_bytes(&bytes).unwrap());
        main_bnl.set_user_data(bnl_arc);

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
            count,
            receiver,
        }
    }

    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::TreeClicked => {
                        if let Some(item) = self.tree.first_selected_item() {
                            let data: Option<Arc<BNLFile>> = unsafe { item.user_data() };

                            match data {
                                Some(data) => {
                                    data.asset_descriptions().iter().for_each(|desc| {
                                        self.tree.insert(&item, desc.name(), i32::MAX);
                                    });
                                }
                                None => eprintln!("No data available."),
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
