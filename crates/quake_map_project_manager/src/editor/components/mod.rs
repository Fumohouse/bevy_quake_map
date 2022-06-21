// Inspired by: https://github.com/jakobhellermann/bevy_editor_pls

use crate::{document::DocumentIoContext, io::EditorIo, project::EditorProject};
use bevy_egui::EguiContext;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

pub mod entity_definition_editor;
pub mod project_panel;

pub struct ComponentDrawContext<'a> {
    pub project: &'a mut EditorProject,
    pub io: Arc<dyn EditorIo>,
    pub doc_context: &'a DocumentIoContext,
    pub component_states: &'a mut ComponentStates,
}

#[derive(Default)]
pub struct ComponentStates(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

impl ComponentStates {
    pub fn insert<T>(&mut self, state: T)
    where
        T: Any + Send + Sync,
    {
        self.0.insert(TypeId::of::<T>(), Box::new(state));
    }

    pub fn get_state<T: Any + Send + Sync>(&self) -> &T {
        self.0[&TypeId::of::<T>()].downcast_ref().unwrap()
    }

    pub fn get_state_mut<T: Any + Send + Sync>(&mut self) -> &mut T {
        self.0
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut()
            .unwrap()
    }
}

pub trait EditorComponent: Send + Sync {
    fn draw(&self, egui_context: &mut EguiContext, component_context: &mut ComponentDrawContext);
}

pub trait EditorComponentWithState: EditorComponent {
    type State: Default + Any + Send + Sync;
}
