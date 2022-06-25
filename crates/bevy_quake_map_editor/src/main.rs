use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use rfd::FileDialog;
use std::path::PathBuf;

mod editor;
use editor::EditorPlugin;

mod document;
mod io;
mod project;

// TODO: Remove this & FromWorld impl (0.8)
struct EditorProjectFolder(PathBuf);

fn pick_folder() -> Option<PathBuf> {
    FileDialog::new()
        .set_directory(dirs::document_dir()?.as_path())
        .pick_folder()
}

fn main() {
    let folder = match pick_folder() {
        Some(folder) => folder,
        None => {
            println!("No folder selected. Exiting...");
            return;
        }
    };

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .insert_resource(EditorProjectFolder(folder))
        .add_plugin(EguiPlugin)
        .add_plugin(EditorPlugin);

    app.run();
}
