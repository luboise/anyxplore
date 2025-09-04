struct EditWindow {
    scroll: Scroll,
    pack: Pack,
}

impl EditWindow {
    pub fn new<T: GroupExt>(parent_widget: &mut T) -> Result<EditWindow, FltkError> {
        let mut scroll = fltk::group::Scroll::default()
            .with_label("Edit Window")
            .with_type(ScrollType::Vertical)
            .with_size(600, 800);

        parent_widget.add(&scroll);

        scroll.begin();

        let mut pack = Pack::default().with_size(scroll.w(), scroll.h());
        // let mut pack = Pack::default();

        pack.begin();

        pack.add(
            &frame::Frame::default_fill(), // .with_size(300, 300),
        );

        pack.set_spacing(2);
        pack.set_type(PackType::Vertical);

        pack.end();

        scroll.add(&pack);

        scroll.end();
        scroll.show();

        Ok(EditWindow { scroll, pack })
    }

    pub fn reset_begin(&mut self) {
        self.pack.clear();
        self.pack.begin();
    }

    pub fn reset_end(&mut self) {
        self.pack.end();

        self.scroll.scroll_to(0, 0);
        self.redraw();
    }

    pub fn redraw(&mut self) {
        self.pack.redraw();
        self.scroll.redraw();
    }

    pub fn add_viewer<V: Viewable>(&mut self, viewable: V) {
        viewable.create_viewer(&mut self.pack);
    }

    pub fn add_editor<E: Editable>(&mut self, editable: &mut E) {
        editable.create_editor(&mut self.pack);
    }

    fn scroll_mut(&mut self) -> &mut Scroll {
        &mut self.scroll
    }

    fn scroll(&self) -> &Scroll {
        &self.scroll
    }
}
