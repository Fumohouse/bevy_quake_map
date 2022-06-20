use super::{widgets::grid_inspector, EditorComponent, EditorComponentContext};
use crate::document::{
    entity::EntityDefinition, DocumentState, EditorDocument, EditorDocumentItem,
};
use bevy_egui::{
    egui::{self, style::Margin, Color32},
    EguiContext,
};
use bevy_quake_map::fgd::{FgdClass, FgdClassType};

#[derive(Default)]
pub struct ProjectPanel {
    new_doc_name: String,
    selected_entity: Option<String>,
}

impl ProjectPanel {
    fn entity_name(
        &mut self,
        ctx: &EditorComponentContext,
        ui: &mut egui::Ui,
        current_entity: Option<&str>,
    ) -> Option<String> {
        const SPACING: f32 = 4.0;

        let name_is_taken =
            if let Some(project) = ctx.read_project().entities.get(&self.new_doc_name) {
                Some(project.read().name()) != current_entity
            } else {
                false
            };

        let mut name = None;

        egui::Frame::none()
            .inner_margin(Margin::same(SPACING))
            .show(ui, |ui| {
                ui.label("New FGD class name:");
                ui.add_space(SPACING);

                if ui.text_edit_singleline(&mut self.new_doc_name).lost_focus() && !name_is_taken {
                    let new_name = self.new_doc_name.clone();
                    self.new_doc_name.clear();

                    name = Some(new_name);
                    ui.close_menu();
                }

                if name_is_taken {
                    ui.add_space(SPACING);
                    ui.horizontal_wrapped(|ui| {
                        ui.colored_label(Color32::RED, "This entity name is already in use.");
                    });
                }
            });

        name
    }

    fn project_settings(&mut self, ctx: &EditorComponentContext, ui: &mut egui::Ui) {
        let project = ctx.read_project();
        let mut settings = project.settings.write();

        ui.collapsing("Project Settings", |ui| {
            grid_inspector("project_settings", ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut settings.name);
                ui.end_row();

                ui.label("Description");
                ui.text_edit_multiline(&mut settings.description);
                ui.end_row();
            });
        });
    }

    fn entity_selector(&mut self, ctx: &EditorComponentContext, ui: &mut egui::Ui) {
        ui.collapsing("Entity Definitions", |ui| {
            let mut to_rename = None;
            let mut to_remove = None;

            for (name, doc) in ctx.read_project().entities.iter() {
                let response = ui.add(egui::SelectableLabel::new(
                    self.selected_entity.as_deref() == Some(name),
                    name,
                ));

                if response.clicked() {
                    self.selected_entity = Some(name.to_owned());
                }

                response.context_menu(|ui| {
                    ui.menu_button("Rename", |ui| {
                        if let Some(new_name) = self.entity_name(ctx, ui, Some(name)) {
                            to_rename = Some((doc.clone(), new_name.clone()));
                            self.new_doc_name.clear();
                            ui.close_menu();

                            if Some(name) == self.selected_entity.as_ref() {
                                self.selected_entity = Some(new_name);
                            }
                        }
                    });

                    if ui.button("Delete").clicked() {
                        to_remove = Some(name.to_owned());
                        ui.close_menu();
                    }
                });
            }

            if let Some((doc, new_name)) = to_rename {
                ctx.write_project().entities.rename(&doc, &new_name);
                // TODO: Move to a Task (?) + better error handling
                doc.save(ctx.io.as_ref(), ctx.doc_context).unwrap();
            }

            if let Some(name) = to_remove {
                let doc = ctx.write_project().entities.remove(&name).unwrap();
                // TODO: Move to a task (?) + better error handling
                doc.delete(ctx.io.as_ref()).unwrap();
            }

            ui.menu_button("+ Add", |ui| {
                if let Some(new_name) = self.entity_name(ctx, ui, None) {
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

                    ctx.write_project().entities.insert(doc);
                    self.selected_entity = Some(new_name);
                }
            });
        });
    }
}

impl EditorComponent for ProjectPanel {
    fn draw(&mut self, egui_context: &mut EguiContext, component_context: &EditorComponentContext) {
        egui::SidePanel::left("project_panel")
            .default_width(250.0)
            .width_range(100.0..=400.0)
            .show(egui_context.ctx_mut(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Project");
                });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.project_settings(component_context, ui);
                    self.entity_selector(component_context, ui);
                });
            });
    }
}
