use super::{EditorComponent, EditorComponentContext};
use crate::document::{entity::EntityDefinition, EditorDocument, DocumentState};
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

        let name_is_taken = ctx.project.read().unwrap().entities.iter().any(|e| {
            let name = &e.read().class.name;
            name == &self.new_doc_name && Some(name as &str) != current_entity
        });

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

    fn entity_selector(&mut self, ctx: &EditorComponentContext, ui: &mut egui::Ui) {
        ui.collapsing("Entities", |ui| {
            let mut to_remove = None;

            for (idx, doc) in ctx.project.read().unwrap().entities.iter().enumerate() {
                let name = doc.read().class.name.clone();

                let response = ui.add(egui::SelectableLabel::new(
                    self.selected_entity.as_ref() == Some(&name),
                    &name,
                ));

                if response.clicked() {
                    self.selected_entity = Some(name.clone());
                }

                response.context_menu(|ui| {
                    ui.menu_button("Rename", |ui| {
                        if let Some(new_name) = self.entity_name(ctx, ui, Some(&name)) {
                            doc.rename(&new_name);
                            // TODO: Move to a Task (?) + better error handling
                            doc.save(ctx.io.as_ref(), ctx.doc_context).unwrap();
                            self.new_doc_name.clear();
                            ui.close_menu();

                            if Some(name) == self.selected_entity {
                                self.selected_entity = Some(new_name);
                            }
                        }
                    });

                    if ui.button("Delete").clicked() {
                        // TODO: Move to a task (?) + better error handling
                        doc.delete(ctx.io.as_ref()).unwrap();
                        to_remove = Some(idx);
                        ui.close_menu();
                    }
                });
            }

            if let Some(idx) = to_remove {
                ctx.project.write().unwrap().entities.remove(idx);
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

                    ctx.project.write().unwrap().entities.push(doc);
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
                    self.entity_selector(component_context, ui);
                });
            });
    }
}
