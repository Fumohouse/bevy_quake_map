use crate::io::{EditorIo, EditorIoError};
use bevy::{prelude::*, reflect::TypeRegistryArc};
use std::{
    path::Path,
    str::Utf8Error,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use thiserror::Error;

pub mod entity;

#[derive(Clone)]
pub struct DocumentIoContext {
    pub type_registry: TypeRegistryArc,
}

impl FromWorld for DocumentIoContext {
    fn from_world(world: &mut World) -> Self {
        Self {
            type_registry: world.resource::<TypeRegistryArc>().clone(),
        }
    }
}

#[derive(Error, Debug)]
pub enum DocumentIoError {
    #[error("ron error: {0}")]
    Ron(#[from] ron::Error),
    #[error("utf8 error: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("editor io error: {0}")]
    EditorIo(#[from] EditorIoError),
}

#[derive(Clone)]
pub enum DocumentState {
    New,
    Clean,
    Modified,
    Renamed(String),
}

pub trait EditorDocumentItem: Sized {
    fn deserialize(
        serialized: &str,
        doc_context: &DocumentIoContext,
    ) -> Result<Self, DocumentIoError>;

    fn serialize(&self, doc_context: &DocumentIoContext) -> Result<String, DocumentIoError>;

    fn save_path(&self) -> String;

    fn set_name(&mut self, new_name: &str);
}

pub struct EditorDocument<T: EditorDocumentItem> {
    internal: Arc<RwLock<T>>,
    state: Arc<RwLock<DocumentState>>,
}

impl<T: EditorDocumentItem> Clone for EditorDocument<T> {
    fn clone(&self) -> Self {
        Self {
            internal: self.internal.clone(),
            state: self.state.clone(),
        }
    }
}

impl<T: EditorDocumentItem> EditorDocument<T> {
    pub fn new(item: T, initial_state: DocumentState) -> Self {
        Self {
            internal: Arc::new(RwLock::new(item)),
            state: Arc::new(RwLock::new(initial_state)),
        }
    }

    pub fn state(&self) -> RwLockReadGuard<DocumentState> {
        self.state.read().unwrap()
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.internal.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        // Lock does not get dropped when put into if let.
        let current_state = self.state().clone();

        if let DocumentState::Clean = current_state {
            *self.state.write().unwrap() = DocumentState::Modified;
        }

        self.internal.write().unwrap()
    }

    pub fn load_buf(
        serialized: &[u8],
        doc_context: &DocumentIoContext,
    ) -> Result<Self, DocumentIoError> {
        let content_string = std::str::from_utf8(serialized)?;
        Self::load(content_string, doc_context)
    }

    pub fn load(
        serialized: &str,
        doc_context: &DocumentIoContext,
    ) -> Result<Self, DocumentIoError> {
        T::deserialize(serialized, doc_context).map(|item| Self::new(item, DocumentState::Clean))
    }

    pub fn save(
        &self,
        io: &dyn EditorIo,
        doc_context: &DocumentIoContext,
    ) -> Result<(), DocumentIoError> {
        if let DocumentState::Clean = *self.state() {
            return Ok(());
        }

        if let DocumentState::Renamed(old_path) = &*self.state() {
            io.delete_file(Path::new(old_path))?;
        }

        let res = self.read().serialize(doc_context)?;
        io.write_file(Path::new(&self.read().save_path()), res.as_bytes())?;

        *self.state.write().unwrap() = DocumentState::Clean;
        Ok(())
    }

    pub fn rename(&self, new_name: &str) {
        let old_path = self.read().save_path();

        self.write().set_name(new_name);

        // New files don't have an old path;
        // don't overwrite old path if renamed multiple times
        if let DocumentState::New | DocumentState::Renamed(_) = *self.state() {
            return;
        }

        *self.state.write().unwrap() = DocumentState::Renamed(old_path);
    }

    pub fn delete(&self, io: &dyn EditorIo) -> Result<(), DocumentIoError> {
        if let DocumentState::New = *self.state() {
            return Ok(());
        }

        let path = match &*self.state() {
            DocumentState::Renamed(old_path) => old_path.clone(),
            _ => self.read().save_path(),
        };

        io.delete_file(Path::new(&path))?;

        // Treat future operations (ideally none) as if the document is new
        *self.state.write().unwrap() = DocumentState::New;

        Ok(())
    }
}
