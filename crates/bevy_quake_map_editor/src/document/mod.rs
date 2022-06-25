use crate::io::{EditorIo, EditorIoError};
use bevy::{prelude::*, reflect::TypeRegistryArc};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    collections::HashMap,
    ops::Deref,
    path::Path,
    str::Utf8Error,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use thiserror::Error;

pub mod entity;
pub mod game_settings;

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

    fn name(&self) -> &str;
    fn set_name(&mut self, new_name: &str);
}

static DOC_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(PartialEq, Copy, Clone)]
pub struct DocumentId(usize);

pub struct EditorDocument<T: EditorDocumentItem> {
    id: DocumentId,
    internal: Arc<RwLock<T>>,
    state: Arc<RwLock<DocumentState>>,
}

impl<T: EditorDocumentItem> Clone for EditorDocument<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            internal: self.internal.clone(),
            state: self.state.clone(),
        }
    }
}

impl<T: EditorDocumentItem> EditorDocument<T> {
    pub fn new(item: T, initial_state: DocumentState) -> Self {
        // Similar to bevy handling
        let id = DOC_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                val.checked_add(1)
            })
            .map(DocumentId)
            .expect("Document ID overflow");

        Self {
            id,
            internal: Arc::new(RwLock::new(item)),
            state: Arc::new(RwLock::new(initial_state)),
        }
    }

    pub fn id(&self) -> DocumentId {
        self.id
    }

    pub fn state(&self) -> DocumentState {
        self.state.read().clone()
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.internal.read()
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.internal.write()
    }

    pub fn can_read(&self) -> bool {
        self.internal.try_read().is_some()
    }

    pub fn mark_changed(&self) {
        // Lock does not get dropped when put into if let.
        let current_state = self.state.read().clone();

        if let DocumentState::Clean = current_state {
            *self.state.write() = DocumentState::Modified;
        }
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
        if let DocumentState::Clean = *self.state.read() {
            return Ok(());
        }

        if let DocumentState::Renamed(old_path) = &*self.state.read() {
            io.delete_file(Path::new(old_path))?;
        }

        let res = self.read().serialize(doc_context)?;
        io.write_file(Path::new(&self.read().save_path()), res.as_bytes())?;

        *self.state.write() = DocumentState::Clean;
        Ok(())
    }

    pub fn rename(&self, new_name: &str) {
        let old_path = self.read().save_path();

        self.write().set_name(new_name);

        // New files don't have an old path;
        // don't overwrite old path if renamed multiple times
        if let DocumentState::New | DocumentState::Renamed(_) = *self.state.read() {
            return;
        }

        *self.state.write() = DocumentState::Renamed(old_path);
    }

    pub fn delete(&self, io: &dyn EditorIo) -> Result<(), DocumentIoError> {
        if let DocumentState::New = *self.state.read() {
            return Ok(());
        }

        let path = match &*self.state.read() {
            DocumentState::Renamed(old_path) => old_path.clone(),
            _ => self.read().save_path(),
        };

        io.delete_file(Path::new(&path))?;

        // Treat future operations (ideally none) as if the document is new
        *self.state.write() = DocumentState::New;

        Ok(())
    }
}

pub struct DocumentCollection<T: EditorDocumentItem> {
    internal: HashMap<String, EditorDocument<T>>,
}

impl<T: EditorDocumentItem> Clone for DocumentCollection<T> {
    fn clone(&self) -> Self {
        Self {
            internal: self.internal.clone(),
        }
    }
}

impl<T: EditorDocumentItem> Default for DocumentCollection<T> {
    fn default() -> Self {
        Self {
            internal: HashMap::new(),
        }
    }
}

impl<T: EditorDocumentItem> Deref for DocumentCollection<T> {
    type Target = HashMap<String, EditorDocument<T>>;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<T: EditorDocumentItem> DocumentCollection<T> {
    pub fn remove(&mut self, name: &str) -> Option<EditorDocument<T>> {
        self.internal.remove(name)
    }

    pub fn rename(&mut self, doc: &EditorDocument<T>, to: &str) {
        let doc = self.internal.remove(doc.read().name()).unwrap();
        doc.rename(to);

        self.insert(doc);
    }

    pub fn insert(&mut self, item: EditorDocument<T>) {
        let name = item.read().name().to_owned();
        self.internal.insert(name, item);
    }
}
