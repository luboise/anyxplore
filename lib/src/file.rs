pub trait FileHandle {}

pub trait VirtualFile {
    /*
    fn parent() -> Option<&Arc<impl FileHandle>>;
    */
    fn children(&self) -> &Vec<impl VirtualFile>;

    fn size() -> usize;

    fn is_dirlike() -> bool;
}
