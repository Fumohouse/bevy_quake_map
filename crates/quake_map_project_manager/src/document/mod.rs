use bevy::{prelude::*, reflect::TypeRegistryArc};
use thiserror::Error;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

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
}

pub trait EditorDocumentItem: Sized {
    fn deserialize(serialized: &str, doc_context: &DocumentIoContext) -> Result<Self, DocumentIoError>;
    fn serialize(&self, doc_context: &DocumentIoContext) -> Result<String, DocumentIoError>;
}

pub struct EditorDocument<T: EditorDocumentItem> {
    internal: Arc<RwLock<T>>,
    modified: Arc<RwLock<bool>>,
}

impl<T: EditorDocumentItem> EditorDocument<T> {
    pub fn new(item: T) -> Self {
        Self {
            internal: Arc::new(RwLock::new(item)),
            modified: Arc::new(RwLock::new(false)),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.internal.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        *self.modified.write().unwrap() = true;
        self.internal.write().unwrap()
    }

    pub fn load(serialized: &str, doc_context: &DocumentIoContext) -> Result<Self, DocumentIoError> {
        T::deserialize(serialized, doc_context).map(Self::new)
    }

    pub fn save(&self, doc_context: &DocumentIoContext) -> Result<String, DocumentIoError> {
        *self.modified.write().unwrap() = false;
        self.read().serialize(doc_context)
    }
}
