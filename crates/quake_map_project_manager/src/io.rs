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
    fn delete_file(&self, path: &Path) -> Result<(), EditorIoError>;
    fn move_file(&self, from: &Path, to: &Path) -> Result<(), EditorIoError>;

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, EditorIoError>;

    fn create_directory(&self, path: &Path) -> Result<(), EditorIoError>;

    fn create_dir_if_not_exists(&self, path: &Path) -> Result<(), EditorIoError> {
        match self.read_directory(path) {
            Ok(_) => Ok(()),
            Err(EditorIoError::NotFound(..)) => {
                self.create_directory(path)?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
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

    fn delete_file(&self, path: &Path) -> Result<(), EditorIoError> {
        Ok(fs::remove_file(self.root.join(path))?)
    }

    fn move_file(&self, from: &Path, to: &Path) -> Result<(), EditorIoError> {
        Ok(fs::rename(self.root.join(from), self.root.join(to))?)
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
        Ok(fs::create_dir_all(self.root.join(path))?)
    }
}
