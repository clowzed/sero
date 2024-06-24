use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ArchiveFile {
    pub real_path: PathBuf,
    pub user_path: PathBuf,
}
