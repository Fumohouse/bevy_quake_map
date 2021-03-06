use bevy::math::Vec3;
use bevy_egui::egui::{self, emath::Numeric, style::Margin, Color32};
use std::hash::Hash;

pub mod reorderable_list;

pub fn grid_inspector<H: Hash>(id: H, ui: &mut egui::Ui, draw: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .striped(true)
        .show(ui, draw);
}

pub fn rename_prompt(
    ui: &mut egui::Ui,
    prompt: &str,
    current_value: &mut String,
    is_taken: impl FnOnce(&str) -> bool,
) -> Option<String> {
    const SPACING: f32 = 4.0;

    let name_is_taken = is_taken(current_value);

    let mut name = None;

    egui::Frame::none()
        .inner_margin(Margin::same(SPACING))
        .show(ui, |ui| {
            ui.label(prompt);
            ui.add_space(SPACING);

            if ui.text_edit_singleline(current_value).lost_focus() && !name_is_taken {
                let new_name = current_value.clone();
                current_value.clear();

                name = Some(new_name);
                ui.close_menu();
            }

            if name_is_taken {
                ui.add_space(SPACING);
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(Color32::RED, "This name is already in use.");
                });
            }
        });

    name
}

const ADD_TEXT: &str = "+ Add";

pub fn add_button(ui: &mut egui::Ui) -> egui::Response {
    ui.button(ADD_TEXT)
}

pub fn add_menu(ui: &mut egui::Ui, mut add_contents: impl FnMut(&mut egui::Ui)) {
    ui.menu_button(ADD_TEXT, |ui| {
        add_contents(ui);
    });
}

pub fn num_inspector(ui: &mut egui::Ui, label: &str, num: &mut impl Numeric) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(num))
    })
    .inner
}

pub fn vec3_inspector(ui: &mut egui::Ui, vec: &mut Vec3) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        if num_inspector(ui, "X: ", &mut vec.x).changed() {
            changed = true;
        }

        if num_inspector(ui, "Y: ", &mut vec.y).changed() {
            changed = true;
        }

        if num_inspector(ui, "Z: ", &mut vec.z).changed() {
            changed = true;
        }
    });

    changed
}
