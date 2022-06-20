use bevy_egui::egui;
use std::hash::Hash;

pub fn grid_inspector<H: Hash>(id: H, ui: &mut egui::Ui, draw: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .striped(true)
        .show(ui, draw);
}
