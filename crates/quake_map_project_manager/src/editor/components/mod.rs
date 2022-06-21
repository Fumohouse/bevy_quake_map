// Inspired by: https://github.com/jakobhellermann/bevy_editor_pls

use crate::{document::DocumentIoContext, io::EditorIo, project::EditorProject};
use bevy_egui::EguiContext;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

pub mod project_panel;

pub struct ComponentDrawContext<'a> {
    pub project: Arc<RwLock<EditorProject>>,
    pub io: Arc<dyn EditorIo>,
    pub doc_context: &'a DocumentIoContext,
    pub component_states: ComponentStates,
}

#[derive(Default, Clone)]
pub struct ComponentStates(Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>);

impl ComponentStates {
    pub fn insert<T>(&self, state: T)
    where
        T: Any + Send + Sync,
    {
        self.0.write().insert(TypeId::of::<T>(), Box::new(state));
    }

    pub fn get_state<T: Any + Send + Sync>(&self) -> MappedRwLockReadGuard<T> {
        RwLockReadGuard::map(self.0.read(), |lock| {
            lock[&TypeId::of::<T>()].downcast_ref().unwrap()
        })
    }

    pub fn get_state_mut<T: Any + Send + Sync>(&self) -> MappedRwLockWriteGuard<T> {
        RwLockWriteGuard::map(self.0.write(), |value| {
            value
                .get_mut(&TypeId::of::<T>())
                .unwrap()
                .downcast_mut()
                .unwrap()
        })
    }
}

pub trait EditorComponent: Send + Sync {
    fn draw(&self, egui_context: &mut EguiContext, component_context: &ComponentDrawContext);
}

pub trait EditorComponentWithState: EditorComponent {
    type State: Default + Any + Send + Sync;
}