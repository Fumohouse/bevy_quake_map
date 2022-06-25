use super::{ComponentDrawContext, EditorComponent};
use crate::document::{DocumentId, DocumentState};
use bevy_egui::{egui, EguiContext};

pub mod entity_tab_item;
use entity_tab_item::entity_definition_editor::EntityDefinitionEditorState;

pub trait TabItem: Send + Sync {
    fn id(&self) -> DocumentId;
    fn name(&self) -> String;
    fn state(&self) -> DocumentState;
    fn open(&mut self, component_context: &mut ComponentDrawContext);
    fn draw(&mut self, egui_context: &egui::Context, component_context: &mut ComponentDrawContext);
    fn close(&self, component_context: &mut ComponentDrawContext);
}

#[derive(Default)]
pub struct TabLayoutState {
    tabs: Vec<Box<dyn TabItem>>,
    selected_tab: Option<DocumentId>,
}

impl TabLayoutState {
    pub fn open_or(
        &mut self,
        id: DocumentId,
        component_context: &mut ComponentDrawContext,
        create_item: impl FnOnce() -> Box<dyn TabItem>,
    ) {
        if !self.tabs.iter().any(|item| item.id() == id) {
            let mut tab = create_item();
            tab.open(component_context);

            self.tabs.push(tab);
        }

        self.selected_tab = Some(id);
    }

    pub fn close(&mut self, id: DocumentId, component_context: &mut ComponentDrawContext) {
        let idx = (0..self.tabs.len()).find(|idx| self.tabs[*idx].id() == id);

        if let Some(idx) = idx {
            let tab = self.tabs.remove(idx);

            if Some(tab.id()) == self.selected_tab {
                self.selected_tab = None;
            }

            tab.close(component_context);
        }
    }
}

fn indicator_for(state: &DocumentState) -> &'static str {
    match state {
        DocumentState::New | DocumentState::Modified => "⭕",
        DocumentState::Clean => "✅",
        DocumentState::Renamed(..) => "R",
    }
}

pub struct TabLayout;

impl EditorComponent for TabLayout {
    fn setup(&self, states: &mut super::ComponentStates) {
        states.insert(TabLayoutState::default());
        states.insert(EntityDefinitionEditorState::default());
    }

    fn draw(&self, egui_context: &mut EguiContext, component_context: &mut ComponentDrawContext) {
        let state_ref = component_context
            .component_states
            .get_state::<TabLayoutState>();
        let state = &mut *state_ref.write();

        let mut to_close = None;

        egui::TopBottomPanel::top("main_tabs").show(egui_context.ctx_mut(), |ui| {
            ui.horizontal_wrapped(|ui| {
                for tab in state.tabs.iter() {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                Some(tab.id()) == state.selected_tab,
                                format!("{} - {}", indicator_for(&tab.state()), tab.name()),
                            )
                            .clicked()
                        {
                            state.selected_tab = Some(tab.id());
                        }

                        if ui.button("✖").clicked() {
                            to_close = Some(tab.id());
                        }
                    });
                }

                if let Some(id) = to_close {
                    state.close(id, component_context);
                }
            });
        });

        if let Some(id) = state.selected_tab {
            let tab = state.tabs.iter_mut().find(|tab| tab.id() == id).unwrap();
            tab.draw(egui_context.ctx_mut(), component_context);
        }
    }
}
