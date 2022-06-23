// Inspired by: https://github.com/jakobhellermann/bevy_editor_pls

use crate::{document::DocumentIoContext, io::EditorIo, project::EditorProject};
use bevy::prelude::Commands;
use bevy_egui::EguiContext;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::Arc,
};

pub mod project_panel;
pub mod tab_layout;

pub struct ComponentDrawContext<'a, 'w, 's> {
    pub project: &'a mut EditorProject,
    pub io: Arc<dyn EditorIo>,
    pub doc_context: &'a DocumentIoContext,
    pub component_states: &'a mut ComponentStates,
    pub commands: &'a mut Commands<'w, 's>,
}

pub struct StateRef<T: Any + Send + Sync>(Arc<RwLock<dyn Any + Send + Sync>>, PhantomData<T>);

impl<T: Any + Send + Sync> StateRef<T> {
    pub fn read(&self) -> MappedRwLockReadGuard<T> {
        RwLockReadGuard::map(self.0.read(), |component| component.downcast_ref().unwrap())
    }

    pub fn write(&self) -> MappedRwLockWriteGuard<T> {
        RwLockWriteGuard::map(self.0.write(), |component| {
            component.downcast_mut().unwrap()
        })
    }
}

#[derive(Default)]
pub struct ComponentStates(HashMap<TypeId, Arc<RwLock<dyn Any + Send + Sync>>>);

impl ComponentStates {
    pub fn insert<T>(&mut self, state: T)
    where
        T: Any + Send + Sync,
    {
        self.0
            .insert(TypeId::of::<T>(), Arc::new(RwLock::new(state)));
    }

    pub fn get_state<T: Any + Send + Sync>(&self) -> StateRef<T> {
        StateRef(self.0[&TypeId::of::<T>()].clone(), PhantomData)
    }
}

pub trait EditorComponent: Send + Sync {
    fn setup(&self, states: &mut ComponentStates);
    fn draw(&self, egui_context: &mut EguiContext, component_context: &mut ComponentDrawContext);
}
