use std::{
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MapIoError {
    #[error("file not found: {0}")]
    NotFound(PathBuf),
    #[error("io error: {0}")]
    StdIo(#[from] io::Error),
}

pub trait MapIo: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, MapIoError>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), MapIoError>;
    fn delete_file(&self, path: &Path) -> Result<(), MapIoError>;
    fn move_file(&self, from: &Path, to: &Path) -> Result<(), MapIoError>;

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, MapIoError>;

    fn create_directory(&self, path: &Path) -> Result<(), MapIoError>;

    fn create_dir_if_not_exists(&self, path: &Path) -> Result<(), MapIoError> {
        match self.read_directory(path) {
            Ok(_) => Ok(()),
            Err(MapIoError::NotFound(..)) => {
                self.create_directory(path)?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
