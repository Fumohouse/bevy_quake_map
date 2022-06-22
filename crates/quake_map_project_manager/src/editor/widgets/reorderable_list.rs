use bevy_egui::egui;

#[derive(PartialEq)]
pub enum ItemPosition {
    Only,
    First,
    Middle,
    Last,
}

pub enum ItemAction {
    MoveUp,
    MoveDown,
    Remove,
}

fn apply_movement<T>(idx: usize, movement: ItemAction, list: &mut Vec<T>) -> bool {
    if idx == 0 {
        if let ItemAction::MoveUp = movement {
            return false;
        }
    }

    let item = list.remove(idx);
    let target = match movement {
        ItemAction::MoveUp => idx - 1,
        ItemAction::MoveDown => idx + 1,
        ItemAction::Remove => return true,
    };

    list.insert(target, item);

    true
}

pub fn reorderable_list<T>(
    ui: &mut egui::Ui,
    list: &mut Vec<T>,
    mut draw_item: impl FnMut(&mut egui::Ui, ItemPosition, &mut T) -> Option<ItemAction>,
) -> bool {
    let mut movement = None;
    let size = list.len();

    for (idx, item) in list.iter_mut().enumerate() {
        let position = if size == 1 {
            ItemPosition::Only
        } else if idx == 0 {
            ItemPosition::First
        } else if idx == size - 1 {
            ItemPosition::Last
        } else {
            ItemPosition::Middle
        };

        if let Some(movement_type) = draw_item(ui, position, item) {
            movement = Some((idx, movement_type));
        }
    }

    let mut modified = false;

    if let Some((idx, movement)) = movement {
        modified = apply_movement(idx, movement, list);
    }

    modified
}

pub fn reorderable_list_item(
    position: ItemPosition,
    ui: &mut egui::Ui,
    mut add_contents: impl FnMut(&mut egui::Ui),
) -> Option<ItemAction> {
    let mut movement = None;

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(), |ui| {
            if ui.button("✖").clicked() {
                movement = Some(ItemAction::Remove);
            }

            if position != ItemPosition::Only {
                if position != ItemPosition::Last && ui.button("⬇").clicked() {
                    movement = Some(ItemAction::MoveDown);
                }

                if position != ItemPosition::First && ui.button("⬆").clicked() {
                    movement = Some(ItemAction::MoveUp);
                }
            }

            ui.add_sized(ui.available_size(), |ui: &mut egui::Ui| {
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    add_contents(ui);
                }).response
            });
        });
    });

    movement
}
