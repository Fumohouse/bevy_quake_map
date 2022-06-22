use super::{
    project_panel::ProjectPanelState, ComponentDrawContext, EditorComponent,
    EditorComponentWithState,
};
use crate::{
    document::{entity::EntityDefinition, EditorDocument, EditorDocumentItem},
    editor::widgets,
    project::EditorProject,
};
use bevy::math::{UVec3, Vec3};
use bevy_egui::{
    egui::{self, style::Margin},
    EguiContext,
};
use bevy_quake_map::fgd::{FgdClassProperty, FgdClassType};
use parking_lot::RwLockReadGuard;

#[derive(Default)]
pub struct EntityDefinitionEditorState {
    selected_document: Option<EditorDocument<EntityDefinition>>,
}

pub struct EntityDefinitionEditor;

fn inspect_class_prop(
    ui: &mut egui::Ui,
    project: &EditorProject,
    property: &mut FgdClassProperty,
) -> bool {
    let mut changed = false;

    match property {
        FgdClassProperty::Base(ref mut list) => {
            ui.label("Base classes:");

            changed =
                widgets::reorderable_list::reorderable_list(ui, list, |ui, position, item| {
                    widgets::reorderable_list::reorderable_list_item(position, ui, |ui| {
                        ui.label(&*item);
                    })
                });

            widgets::add_button(ui, |ui| {
                let mut base_classes = project
                    .entities
                    .iter()
                    .filter(|(_key, doc)| {
                        doc.can_read()
                            && doc.read().class.class_type == FgdClassType::Base
                            && !list.iter().any(|name| &doc.read().class.name == name)
                    })
                    .peekable();

                if base_classes.peek().is_none() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label("No other base classes exist.");
                    });
                } else {
                    let mut to_add = None;

                    for (class_name, _) in base_classes {
                        if ui.button(class_name).clicked() {
                            to_add = Some(class_name.clone());
                            ui.close_menu();
                            changed = true;
                        }
                    }

                    if let Some(to_add) = to_add {
                        list.push(to_add);
                    }
                }
            });
        }
        FgdClassProperty::Model(ref mut path) => {
            ui.label("Model path:");
            if ui.text_edit_singleline(path).changed() {
                changed = true;
            }
        }
        FgdClassProperty::Color(ref mut color) => {
            ui.label("Color:");

            let mut color_arr = (color.as_vec3() / 255.0).to_array();

            if ui.color_edit_button_rgb(&mut color_arr).changed() {
                color.x = (color_arr[0] * 255.0) as u32;
                color.y = (color_arr[1] * 255.0) as u32;
                color.z = (color_arr[2] * 255.0) as u32;

                changed = true;
            }
        }
        FgdClassProperty::Size(ref mut p1, ref mut p2) => {
            ui.label("Point 1:");
            if widgets::vec3_inspector(ui, p1) {
                changed = true;
            }

            ui.label("Point 2:");
            if widgets::vec3_inspector(ui, p2) {
                changed = true;
            }
        }
    }

    changed
}

fn inspector(project: &EditorProject, doc: &EditorDocument<EntityDefinition>, ui: &mut egui::Ui) {
    let def = &mut *doc.write();

    widgets::grid_inspector("entity_definition_inspector", ui, |ui| {
        ui.label("Description");
        if ui.text_edit_multiline(&mut def.class.description).changed() {
            doc.mark_changed();
        }

        ui.end_row();

        ui.label("Class Type");
        egui::ComboBox::from_id_source("entity_definition_class_type")
            .selected_text(format!("{:?}", def.class.class_type))
            .show_ui(ui, |ui| {
                let class_type = &mut def.class.class_type;
                let mut changed = false;

                // NOTE: Must sync with the FGD module
                changed = changed
                    || ui
                        .selectable_value(class_type, FgdClassType::Base, "Base")
                        .changed();
                changed = changed
                    || ui
                        .selectable_value(class_type, FgdClassType::Point, "Point")
                        .changed();
                changed = changed
                    || ui
                        .selectable_value(class_type, FgdClassType::Solid, "Solid")
                        .changed();

                if changed {
                    doc.mark_changed();
                }
            });

        ui.end_row();
    });

    ui.separator();

    ui.collapsing("Class Properties", |ui| {
        let class_properties = &mut def.class.class_properties;

        let mut props_changed = false;

        let mut list_changed = widgets::reorderable_list::reorderable_list(
            ui,
            class_properties,
            |ui, position, item| {
                let mut movement = None;

                egui::Frame::canvas(ui.style())
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        movement =
                            widgets::reorderable_list::reorderable_list_item(position, ui, |ui| {
                                ui.label(item.serialize());
                            });

                        ui.separator();

                        props_changed = props_changed || inspect_class_prop(ui, project, item);
                    });

                movement
            },
        );

        widgets::add_button(ui, |ui| {
            ui.label("Select a property type:");

            let mut clicked = false;

            if ui.button("base").clicked() {
                class_properties.push(FgdClassProperty::Base(Vec::new()));
                clicked = true;
            }

            if ui.button("model").clicked() {
                class_properties.push(FgdClassProperty::Model("path/to/model.obj".to_string()));
                clicked = true;
            }

            if ui.button("color").clicked() {
                class_properties.push(FgdClassProperty::Color(UVec3::new(255, 255, 255)));
                clicked = true;
            }

            if ui.button("size").clicked() {
                class_properties.push(FgdClassProperty::Size(
                    Vec3::new(-32.0, -32.0, -32.0),
                    Vec3::new(32.0, 32.0, 32.0),
                ));
                clicked = true;
            }

            if clicked {
                list_changed = true;
                ui.close_menu();
            }
        });

        if props_changed || list_changed {
            doc.mark_changed();
        }
    });

    ui.collapsing("Entity Properties", |ui| {
        ui.label("TODO");
    });
}

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
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        inspector(component_context.project, doc, ui);
                    });
                });
        }
    }
}

impl EditorComponentWithState for EntityDefinitionEditor {
    type State = EntityDefinitionEditorState;
}
