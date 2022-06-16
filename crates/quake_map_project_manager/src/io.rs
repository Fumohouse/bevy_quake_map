use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditorIoError {
    #[error("file not found: {0}")]
    NotFound(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

pub trait EditorIo: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, EditorIoError>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), EditorIoError>;

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, EditorIoError>;

    fn create_directory(&self, path: &Path) -> Result<(), EditorIoError>;
}

pub struct FileEditorIo {
    root: PathBuf,
}

impl FileEditorIo {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
}

impl EditorIo for FileEditorIo {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, EditorIoError> {
        fs::read(self.root.join(path)).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => EditorIoError::NotFound(path.to_path_buf()),
            _ => err.into(),
        })
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), EditorIoError> {
        Ok(fs::write(self.root.join(path), contents)?)
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, EditorIoError> {
        Ok(Box::new(
            fs::read_dir(self.root.join(path))
                .map_err(|err| match err.kind() {
                    io::ErrorKind::NotFound => EditorIoError::NotFound(path.to_path_buf()),
                    _ => err.into(),
                })?
                .map(|entry| entry.unwrap().path()),
        ))
    }

    fn create_directory(&self, path: &Path) -> Result<(), EditorIoError> {
        Ok(fs::create_dir_all(path)?)
    }
}
