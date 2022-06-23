use super::{ComponentDrawContext, EditorComponent, EditorComponentWithState};
use crate::{
    document::{entity::EntityDefinition, DocumentState, EditorDocument, EditorDocumentItem},
    editor::widgets,
};
use bevy_egui::{egui, EguiContext};
use bevy_quake_map::fgd::{FgdClass, FgdClassType};

const FGD_NAME_PROMPT: &str = "New FGD class name:";

#[derive(Default)]
pub struct ProjectPanelState {
    new_doc_name: String,
    pub selected_entity: Option<String>,
}

pub struct ProjectPanel;

fn project_settings(ctx: &ComponentDrawContext, ui: &mut egui::Ui) {
    let settings = &ctx.project.settings;
    let mut settings_doc = settings.write();

    ui.collapsing("Project Settings", |ui| {
        widgets::grid_inspector("project_settings", ui, |ui| {
            ui.label("Name");
            if ui.text_edit_singleline(&mut settings_doc.name).changed() {
                settings.mark_changed();
            }
            ui.end_row();

            ui.label("Description");
            if ui
                .text_edit_multiline(&mut settings_doc.description)
                .changed()
            {
                settings.mark_changed();
            }
            ui.end_row();
        });
    });
}

fn entity_selector(ctx: &mut ComponentDrawContext, ui: &mut egui::Ui) {
    let state = ctx.component_states.get_state_mut::<ProjectPanelState>();

    ui.collapsing("Entity Definitions", |ui| {
        let mut to_rename = None;
        let mut to_remove = None;

        for (name, doc) in ctx.project.entities.iter() {
            let response = ui.add(egui::SelectableLabel::new(
                state.selected_entity.as_deref() == Some(name),
                name,
            ));

            if response.clicked() {
                if state.selected_entity.as_ref() == Some(name) {
                    state.selected_entity = None;
                } else {
                    state.selected_entity = Some(name.to_owned());
                }
            }

            response.context_menu(|ui| {
                ui.menu_button("Rename", |ui| {
                    if let Some(new_name) = widgets::rename_prompt(
                        ui,
                        FGD_NAME_PROMPT,
                        &mut state.new_doc_name,
                        |name| {
                            if let Some(document) = ctx.project.entities.get(name) {
                                Some(document.read().name()) != state.selected_entity.as_deref()
                            } else {
                                false
                            }
                        },
                    ) {
                        to_rename = Some((doc.clone(), new_name.clone()));
                        state.new_doc_name.clear();
                        ui.close_menu();

                        if Some(name) == state.selected_entity.as_ref() {
                            state.selected_entity = Some(new_name);
                        }
                    }
                });

                if ui.button("Delete").clicked() {
                    if Some(name) == state.selected_entity.as_ref() {
                        state.selected_entity = None;
                    }

                    to_remove = Some(name.to_owned());
                    ui.close_menu();
                }
            });
        }

        if let Some((doc, new_name)) = to_rename {
            ctx.project.entities.rename(&doc, &new_name);
            // TODO: Move to a Task (?) + better error handling
            doc.save(ctx.io.as_ref(), ctx.doc_context).unwrap();
        }

        if let Some(name) = to_remove {
            let doc = ctx.project.entities.remove(&name).unwrap();
            // TODO: Move to a task (?) + better error handling
            doc.delete(ctx.io.as_ref()).unwrap();
        }

        widgets::add_menu(ui, |ui| {
            if let Some(new_name) =
                widgets::rename_prompt(ui, FGD_NAME_PROMPT, &mut state.new_doc_name, |name| {
                    ctx.project.entities.contains_key(name)
                })
            {
                let def = EntityDefinition {
                    class: FgdClass {
                        class_type: FgdClassType::Point,
                        name: new_name.clone(),
                        description: "".to_string(),
                        class_properties: vec![],
                        entity_properties: vec![],
                    },
                    scene: None,
                };

                let doc = EditorDocument::new(def, DocumentState::New);

                ctx.project.entities.insert(doc);
                state.selected_entity = Some(new_name);
            }
        });
    });
}

impl EditorComponent for ProjectPanel {
    fn draw(&self, egui_context: &mut EguiContext, component_context: &mut ComponentDrawContext) {
        egui::SidePanel::left("project_panel")
            .default_width(250.0)
            .width_range(100.0..=400.0)
            .show(egui_context.ctx_mut(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Project");
                });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    project_settings(component_context, ui);
                    entity_selector(component_context, ui);
                });
            });
    }
}

impl EditorComponentWithState for ProjectPanel {
    type State = ProjectPanelState;
}
