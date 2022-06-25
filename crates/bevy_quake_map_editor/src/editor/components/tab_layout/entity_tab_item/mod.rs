use super::TabItem;
use crate::{
    document::{
        entity::EntityDefinition, DocumentId, DocumentState, EditorDocument, EditorDocumentItem,
    },
    editor::components::ComponentDrawContext,
};
use bevy_egui::egui;

pub mod entity_definition_editor;

pub struct EntityTabItem {
    pub document: EditorDocument<EntityDefinition>,
}

impl TabItem for EntityTabItem {
    fn id(&self) -> DocumentId {
        self.document.id()
    }

    fn name(&self) -> String {
        self.document.read().save_path()
    }

    fn state(&self) -> DocumentState {
        self.document.state()
    }

    fn open(&mut self, _component_context: &mut ComponentDrawContext) {}

    fn draw(&mut self, egui_context: &egui::Context, component_context: &mut ComponentDrawContext) {
        entity_definition_editor::draw(egui_context, &self.document, component_context);
    }

    fn close(&self, _component_context: &mut ComponentDrawContext) {}
}
