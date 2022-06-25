use bevy_quake_map_editor_common::io::{MapIo, MapIoError};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

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

impl MapIo for FileEditorIo {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, MapIoError> {
        fs::read(self.root.join(path)).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => MapIoError::NotFound(path.to_path_buf()),
            _ => err.into(),
        })
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), MapIoError> {
        Ok(fs::write(self.root.join(path), contents)?)
    }

    fn delete_file(&self, path: &Path) -> Result<(), MapIoError> {
        Ok(fs::remove_file(self.root.join(path))?)
    }

    fn move_file(&self, from: &Path, to: &Path) -> Result<(), MapIoError> {
        Ok(fs::rename(self.root.join(from), self.root.join(to))?)
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, MapIoError> {
        Ok(Box::new(
            fs::read_dir(self.root.join(path))
                .map_err(|err| match err.kind() {
                    io::ErrorKind::NotFound => MapIoError::NotFound(path.to_path_buf()),
                    _ => err.into(),
                })?
                .map(|entry| entry.unwrap().path()),
        ))
    }

    fn create_directory(&self, path: &Path) -> Result<(), MapIoError> {
        Ok(fs::create_dir_all(self.root.join(path))?)
    }
}
