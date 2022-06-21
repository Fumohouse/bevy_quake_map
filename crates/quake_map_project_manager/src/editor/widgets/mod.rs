use bevy_egui::egui::{self, style::Margin, Color32};
use std::hash::Hash;

pub fn grid_inspector<H: Hash>(id: H, ui: &mut egui::Ui, draw: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .striped(true)
        .show(ui, draw);
}

pub fn rename_prompt(
    prompt: &str,
    current_value: &mut String,
    is_taken: impl FnOnce(&str) -> bool,
    ui: &mut egui::Ui,
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
                    ui.colored_label(Color32::RED, "This entity name is already in use.");
                });
            }
        });

    name
}
