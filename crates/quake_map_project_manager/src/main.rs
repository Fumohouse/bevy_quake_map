use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, TaskPool},
};
use bevy_egui::{
    egui::{self, Align2},
    EguiContext, EguiPlugin,
};
use futures_lite::future;
use rfd::FileDialog;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

mod io;
use io::{EditorIo, EditorIoError, FileEditorIo};

mod project;
use project::EditorProject;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum EditorState {
    Loading,
    Saving,
    Ready,
}

// TODO: Remove this & FromWorld impl (0.8)
struct EditorProjectFolder(PathBuf);

struct EditorContext {
    io: Arc<dyn EditorIo>,
    project: Option<Arc<RwLock<EditorProject>>>,
    task_pool: TaskPool,
}

impl FromWorld for EditorContext {
    fn from_world(world: &mut World) -> Self {
        let root = world.resource::<EditorProjectFolder>().0.clone();
        let task_pool = world.resource::<IoTaskPool>().0.clone();

        Self {
            io: Arc::new(FileEditorIo::new(&root)),
            project: None,
            task_pool,
        }
    }
}

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

    app.add_state(EditorState::Loading)
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(EditorProjectFolder(folder))
        .init_resource::<EditorContext>();

    app.add_system_set(SystemSet::on_enter(EditorState::Loading).with_system(begin_load))
        .add_system_set(SystemSet::on_update(EditorState::Loading).with_system(poll_load));

    app.add_system_set(
        SystemSet::on_enter(EditorState::Saving)
            .with_system(prepare_save.exclusive_system().at_start())
            .with_system(begin_save),
    )
    .add_system_set(SystemSet::on_update(EditorState::Saving).with_system(poll_save));

    app.add_system(draw_editor);

    app.run();
}

fn begin_load(mut commands: Commands, editor_context: ResMut<EditorContext>) {
    let io = editor_context.io.clone();

    let task = editor_context
        .task_pool
        .spawn(async move { EditorProject::load(io.as_ref()) });

    commands.spawn().insert(task);
}

fn poll_load(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State<EditorState>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Task<EditorProject>)>,
    mut editor_context: ResMut<EditorContext>,
) {
    egui::Window::new("Please wait...")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spinner();
                ui.label("Loading project...");
            });
        });

    for (entity, mut task) in query.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut *task)) {
            editor_context.project = Some(Arc::new(RwLock::new(result)));

            state.set(EditorState::Ready).unwrap();
            commands.entity(entity).despawn();
        }
    }
}

fn prepare_save(world: &mut World) {
    // TODO
}

fn begin_save(mut commands: Commands, editor_context: Res<EditorContext>) {
    let io = editor_context.io.clone();
    let project = editor_context.project.as_ref().unwrap().clone();

    let task: Task<Result<(), EditorIoError>> = editor_context
        .task_pool
        .spawn(async move { project.read().unwrap().save(io.as_ref()) });

    commands.spawn().insert(task);
}

fn poll_save(
    mut state: ResMut<State<EditorState>>,
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut query: Query<(Entity, &mut Task<Result<(), EditorIoError>>)>,
) {
    egui::Window::new("Saving...")
        .anchor(Align2::RIGHT_BOTTOM, egui::Vec2::new(-20.0, -20.0))
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spinner();
                ui.label("Saving project...");
            });
        });

    for (entity, mut task) in query.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut *task)) {
            match result {
                Ok(_) => {
                    state.pop().unwrap();
                    commands.entity(entity).despawn();
                }
                Err(err) => error!("Failed to save project: {}", err),
            }
        }
    }
}

fn draw_editor(
    mut egui_context: ResMut<EguiContext>,
    mut editor_state: ResMut<State<EditorState>>,
) {
    if let EditorState::Loading = editor_state.current() {
        return;
    }

    egui::TopBottomPanel::top("menu_bar").show(egui_context.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                ui.group(|ui| {
                    ui.set_enabled(!matches!(editor_state.current(), EditorState::Saving));

                    if ui.button("Save").clicked() {
                        editor_state.push(EditorState::Saving).unwrap();
                        ui.close_menu();
                    }
                });
            });
        });
    });
}
