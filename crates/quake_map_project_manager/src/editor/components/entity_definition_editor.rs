use super::{
    project_panel::ProjectPanelState, ComponentDrawContext, EditorComponent,
    EditorComponentWithState,
};
use crate::document::{entity::EntityDefinition, EditorDocument, EditorDocumentItem};
use bevy_egui::{egui, EguiContext};
use parking_lot::RwLockReadGuard;

#[derive(Default)]
pub struct EntityDefinitionEditorState {
    selected_document: Option<EditorDocument<EntityDefinition>>,
}

pub struct EntityDefinitionEditor;

impl EditorComponent for EntityDefinitionEditor {
    fn draw(&self, egui_context: &mut EguiContext, component_context: &mut ComponentDrawContext) {
        let selected_entity = component_context
            .component_states
            .get_state::<ProjectPanelState>()
            .selected_entity
            .clone();

        let mut state = component_context
            .component_states
            .get_state_mut::<EntityDefinitionEditorState>();

        if state
            .selected_document
            .as_ref()
            .map(|doc| RwLockReadGuard::map(doc.read(), |doc| doc.name()))
            .as_deref()
            != selected_entity.as_deref()
        {
            state.selected_document = selected_entity.as_ref().map(|name| {
                component_context
                    .project
                    .read()
                    .entities
                    .get(name)
                    .unwrap()
                    .clone()
            });
        }

        if let Some(doc) = state.selected_document.as_ref() {
            egui::Window::new(format!("Entity Definition: {}", doc.read().name()))
                .id(egui::Id::new("entity_definition_editor"))
                .show(egui_context.ctx_mut(), |ui| {
                    ui.label("TODO");
                });
        }
    }
}

impl EditorComponentWithState for EntityDefinitionEditor {
    type State = EntityDefinitionEditorState;
}
