use crate::{
    document::{entity::EntityDefinition, EditorDocument, EditorDocumentItem},
    editor::{components::ComponentDrawContext, widgets},
    project::EditorProject,
};
use bevy::math::{UVec3, Vec3};
use bevy_egui::egui::{self, style::Margin, InnerResponse};
use bevy_quake_map::fgd::{
    Choice, EntityProperty, EntityPropertyData, FgdClassProperty, FgdClassType, Flag, FlagsData,
    ToFgdLiteral,
};

const INSPECTOR_MARGIN: f32 = 8.0;

#[derive(Default)]
pub struct EntityDefinitionEditorState {
    new_property_name: String,
}

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

            widgets::add_menu(ui, |ui| {
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

fn class_property_inspector(
    ui: &mut egui::Ui,
    doc: &EditorDocument<EntityDefinition>,
    project: &EditorProject,
    def: &mut EntityDefinition,
) {
    let class_properties = &mut def.class.class_properties;

    let mut props_changed = false;

    let mut list_changed =
        widgets::reorderable_list::reorderable_list(ui, class_properties, |ui, position, item| {
            let mut action = None;

            egui::Frame::canvas(ui.style())
                .inner_margin(Margin::same(INSPECTOR_MARGIN))
                .show(ui, |ui| {
                    action = widgets::reorderable_list::reorderable_list_item(position, ui, |ui| {
                        ui.label(item.serialize());
                    });

                    ui.separator();

                    if inspect_class_prop(ui, project, item) {
                        props_changed = true;
                    }
                });

            action
        });

    // NOTE: Must keep length and items in sync with FGD module
    if class_properties.len() < 4 {
        widgets::add_menu(ui, |ui| {
            ui.label("Select a property type:");

            let mut clicked = false;

            if !class_properties
                .iter()
                .any(|p| matches!(p, FgdClassProperty::Base(..)))
                && ui.button("base").clicked()
            {
                class_properties.push(FgdClassProperty::Base(Vec::new()));
                clicked = true;
            }

            if !class_properties
                .iter()
                .any(|p| matches!(p, FgdClassProperty::Model(..)))
                && ui.button("model").clicked()
            {
                class_properties.push(FgdClassProperty::Model("path/to/model.obj".to_string()));
                clicked = true;
            }

            if !class_properties
                .iter()
                .any(|p| matches!(p, FgdClassProperty::Color(..)))
                && ui.button("color").clicked()
            {
                class_properties.push(FgdClassProperty::Color(UVec3::new(255, 255, 255)));
                clicked = true;
            }

            if !class_properties
                .iter()
                .any(|p| matches!(p, FgdClassProperty::Size(..)))
                && ui.button("size").clicked()
            {
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
    }

    if props_changed || list_changed {
        doc.mark_changed();
    }
}

fn inspect_prop_data<T: ToFgdLiteral + Default>(
    ui: &mut egui::Ui,
    mut default_inspector: impl FnMut(&mut egui::Ui, &mut T) -> bool,
    data: &mut EntityPropertyData<T>,
) -> bool {
    let mut changed = false;

    ui.label("Display name");
    if ui.text_edit_singleline(&mut data.display_name).changed() {
        changed = true;
    }
    ui.end_row();

    ui.label("Description");
    if ui.text_edit_multiline(&mut data.description).changed() {
        changed = true;
    }
    ui.end_row();

    ui.label("Default value");
    if default_inspector(ui, &mut data.default) {
        changed = true;
    }
    ui.end_row();

    changed
}

fn select_prop_type(
    ui: &mut egui::Ui,
    label: &str,
    current: &mut EntityProperty,
    is_this_variant: bool,
    create: impl FnOnce(String) -> EntityProperty,
) -> bool {
    let response = ui.selectable_label(is_this_variant, label);

    if response.clicked() {
        *current = create(current.name().to_owned());
        true
    } else {
        false
    }
}

macro_rules! select_simple_entity_prop_type {
    ($ui:expr, $prop:expr, $variant:ident) => {
        select_prop_type(
            $ui,
            stringify!($variant),
            $prop,
            matches!($prop, EntityProperty::$variant(..)),
            |name| EntityProperty::$variant(EntityPropertyData::named(name)),
        )
    };
}

fn entity_prop_type_dropdown(ui: &mut egui::Ui, prop: &mut EntityProperty) -> InnerResponse<bool> {
    let mut changed = false;

    let res = egui::ComboBox::from_id_source(format!("entity_property_{}_type", prop.name()))
        .selected_text(prop.type_name())
        .show_ui(ui, |ui| {
            // NOTE: Must sync with FGD module
            changed = select_simple_entity_prop_type!(ui, prop, String);
            changed = changed || select_simple_entity_prop_type!(ui, prop, Integer);
            changed = changed || select_simple_entity_prop_type!(ui, prop, Boolean);
            changed = changed || select_simple_entity_prop_type!(ui, prop, Float);

            changed = changed
                || select_prop_type(
                    ui,
                    "Choices",
                    prop,
                    matches!(prop, EntityProperty::Choices(..)),
                    |name| EntityProperty::Choices(EntityPropertyData::named(name), Vec::new()),
                );

            changed = changed
                || select_prop_type(
                    ui,
                    "Flags",
                    prop,
                    matches!(prop, EntityProperty::Flags(..)),
                    |name| {
                        EntityProperty::Flags(FlagsData {
                            name,
                            flags: Vec::new(),
                        })
                    },
                );
        });

    InnerResponse::new(changed, res.response)
}

fn inspect_entity_prop(ui: &mut egui::Ui, prop: &mut EntityProperty) -> bool {
    let prop_name = prop.name().to_owned();
    let mut changed = false;

    widgets::grid_inspector(format!("entity_property_{}", prop_name), ui, |ui| {
        ui.label("Property type");
        changed = entity_prop_type_dropdown(ui, prop).inner;
        ui.end_row();

        let data_changed = match prop {
            EntityProperty::String(ref mut data) => inspect_prop_data(
                ui,
                |ui, value| ui.text_edit_singleline(value).changed(),
                data,
            ),
            EntityProperty::Integer(ref mut data) => inspect_prop_data(
                ui,
                |ui, value| ui.add(egui::DragValue::new(value)).changed(),
                data,
            ),
            EntityProperty::Boolean(ref mut data) => {
                inspect_prop_data(ui, |ui, value| ui.checkbox(value, "").changed(), data)
            }
            EntityProperty::Float(ref mut data) => inspect_prop_data(
                ui,
                |ui, value| ui.add(egui::DragValue::new(value)).changed(),
                data,
            ),
            EntityProperty::Choices(ref mut data, ref mut choices) => {
                let mut changed = inspect_prop_data(
                    ui,
                    |ui, value| {
                        let mut default_changed = false;

                        egui::ComboBox::from_id_source(format!(
                            "entity_property_{}_choices",
                            prop_name
                        ))
                        .selected_text(
                            choices
                                .get(*value as usize)
                                .map(|choice| &choice.name as &str)
                                .unwrap_or_else(|| "Select a default value..."),
                        )
                        .show_ui(ui, |ui| {
                            for choice in choices.iter() {
                                if ui
                                    .selectable_value(value, choice.index, &choice.name)
                                    .changed()
                                {
                                    default_changed = true;
                                }
                            }
                        });

                        default_changed
                    },
                    data,
                );

                ui.label("Choices");
                ui.vertical(|ui| {
                    if widgets::reorderable_list::reorderable_list(
                        ui,
                        choices,
                        |ui, position, item| {
                            widgets::reorderable_list::reorderable_list_item(position, ui, |ui| {
                                if ui.add(egui::DragValue::new(&mut item.index)).changed() {
                                    changed = true;
                                }

                                if ui.text_edit_singleline(&mut item.name).changed() {
                                    changed = true;
                                }
                            })
                        },
                    ) {
                        changed = true;
                    }

                    if widgets::add_button(ui).clicked() {
                        let index = choices.len() as i32;

                        choices.push(Choice {
                            index,
                            name: "Choice name".to_string(),
                        });
                    }
                });

                ui.end_row();

                changed
            }
            EntityProperty::Flags(ref mut data) => {
                ui.label("Flags");

                let mut flags_changed = false;
                let mut list_changed = false;

                ui.vertical(|ui| {
                    list_changed = widgets::reorderable_list::reorderable_list(
                        ui,
                        &mut data.flags,
                        |ui, position, item| {
                            widgets::reorderable_list::reorderable_list_item(position, ui, |ui| {
                                if ui.add(egui::DragValue::new(&mut item.flag)).changed() {
                                    flags_changed = true;
                                }

                                if ui.checkbox(&mut item.default, "").changed() {
                                    flags_changed = true;
                                }

                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut item.name)
                                            .desired_width(f32::INFINITY),
                                    )
                                    .changed()
                                {
                                    flags_changed = true;
                                }
                            })
                        },
                    );

                    if widgets::add_button(ui).clicked() {
                        let flag = 2_i32.pow(data.flags.len() as u32);

                        data.flags.push(Flag {
                            flag,
                            name: "Flag name".to_string(),
                            default: false,
                        });

                        list_changed = true;
                    }
                });

                ui.end_row();

                list_changed || flags_changed
            }
        };

        changed = changed || data_changed;
    });

    changed
}

fn entity_property_inspector(
    ui: &mut egui::Ui,
    new_property_name: &mut String,
    doc: &EditorDocument<EntityDefinition>,
    def: &mut EntityDefinition,
) {
    const ENTITY_PROP_NAME_PROMPT: &str = "New property name:";

    let entity_properties = &mut def.class.entity_properties;

    let taken_names = entity_properties
        .iter()
        .map(|prop| prop.name().to_owned())
        .collect::<Vec<_>>();

    let mut props_changed = false;

    let mut list_changed =
        widgets::reorderable_list::reorderable_list(ui, entity_properties, |ui, position, item| {
            let mut action = None;

            let res = egui::Frame::canvas(ui.style())
                .inner_margin(Margin::same(INSPECTOR_MARGIN))
                .show(ui, |ui| {
                    action = widgets::reorderable_list::reorderable_list_item(position, ui, |ui| {
                        ui.label(item.name());
                    });

                    if inspect_entity_prop(ui, item) {
                        props_changed = true;
                    }
                });

            res.response.context_menu(|ui| {
                ui.menu_button("Rename", |ui| {
                    if let Some(new_name) = widgets::rename_prompt(
                        ui,
                        ENTITY_PROP_NAME_PROMPT,
                        new_property_name,
                        |name| taken_names.iter().any(|n| n == name && n != item.name()),
                    ) {
                        item.set_name(new_name);

                        ui.close_menu();
                        props_changed = true;
                    }
                });
            });

            action
        });

    widgets::add_menu(ui, |ui| {
        if let Some(new_name) =
            widgets::rename_prompt(ui, ENTITY_PROP_NAME_PROMPT, new_property_name, |name| {
                taken_names.iter().any(|n| n == name)
            })
        {
            entity_properties.push(EntityProperty::String(EntityPropertyData::<String> {
                name: new_name,
                display_name: "Display Name".to_string(),
                default: "Default Value".to_string(),
                description: "Description".to_string(),
            }));

            ui.close_menu();
            list_changed = true;
        }
    });

    if props_changed || list_changed {
        doc.mark_changed();
    }
}

fn inspector(
    state: &mut EntityDefinitionEditorState,
    doc: &EditorDocument<EntityDefinition>,
    project: &EditorProject,
    ui: &mut egui::Ui,
) {
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
        class_property_inspector(ui, doc, project, def);
    });

    ui.collapsing("Entity Properties", |ui| {
        entity_property_inspector(ui, &mut state.new_property_name, doc, def);
    });
}

pub fn draw(
    egui_context: &egui::Context,
    doc: &EditorDocument<EntityDefinition>,
    component_context: &mut ComponentDrawContext,
) {
    let state_ref = component_context
        .component_states
        .get_state::<EntityDefinitionEditorState>();

    let state = &mut *state_ref.write();

    egui::Window::new(format!("Entity Definition: {}", doc.read().name()))
        .id(egui::Id::new("entity_definition_editor"))
        .show(egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                inspector(state, doc, component_context.project, ui);
            });
        });
}
