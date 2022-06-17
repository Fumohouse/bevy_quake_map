use crate::{
    document::{DocumentIoContext, DocumentIoError},
    io::{EditorIo, FileEditorIo},
    project::EditorProject,
    EditorProjectFolder,
};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, TaskPool},
};
use bevy_egui::{
    egui::{self, Align2},
    EguiContext,
};
use futures_lite::future;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

mod project_panel;
use project_panel::ProjectPanel;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum EditorState {
    Loading,
    Saving,
    Ready,
}

struct EditorComponentContext<'a> {
    project: Arc<RwLock<EditorProject>>,
    io: Arc<dyn EditorIo>,
    doc_context: &'a DocumentIoContext,
}

impl<'a> EditorComponentContext<'a> {
    pub fn read_project(&self) -> RwLockReadGuard<EditorProject> {
        self.project.read().unwrap()
    }

    pub fn write_project(&self) -> RwLockWriteGuard<EditorProject> {
        self.project.write().unwrap()
    }
}

trait EditorComponent: Send + Sync {
    fn draw(&mut self, egui_context: &mut EguiContext, component_context: &EditorComponentContext);
}

struct EditorContext {
    io: Arc<dyn EditorIo>,
    project: Option<Arc<RwLock<EditorProject>>>,
    task_pool: TaskPool,
    components: Vec<Box<dyn EditorComponent>>,
}

impl EditorContext {
    fn project(&self) -> Arc<RwLock<EditorProject>> {
        self.project.as_ref().unwrap().clone()
    }
}

impl FromWorld for EditorContext {
    fn from_world(world: &mut World) -> Self {
        let root = world.resource::<EditorProjectFolder>().0.clone();
        let task_pool = world.resource::<IoTaskPool>().0.clone();

        Self {
            io: Arc::new(FileEditorIo::new(&root)),
            project: None,
            task_pool,
            components: vec![Box::new(ProjectPanel::default())],
        }
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(EditorState::Loading)
            .init_resource::<DocumentIoContext>()
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
    }
}

fn begin_load(
    mut commands: Commands,
    editor_context: Res<EditorContext>,
    doc_context: Res<DocumentIoContext>,
) {
    let io = editor_context.io.clone();
    let doc_context = doc_context.clone();

    let task = editor_context
        .task_pool
        .spawn(async move { EditorProject::load(io.as_ref(), doc_context) });

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

fn begin_save(
    mut commands: Commands,
    editor_context: Res<EditorContext>,
    doc_context: Res<DocumentIoContext>,
) {
    let io = editor_context.io.clone();
    let project = editor_context.project();

    let doc_context = doc_context.clone();

    let task = editor_context
        .task_pool
        .spawn(async move { project.read().unwrap().save(io.as_ref(), doc_context) });

    commands.spawn().insert(task);
}

fn poll_save(
    mut state: ResMut<State<EditorState>>,
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut query: Query<(Entity, &mut Task<Result<(), DocumentIoError>>)>,
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
    mut editor_context: ResMut<EditorContext>,
    doc_context: Res<DocumentIoContext>,
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

    let component_ctx = EditorComponentContext {
        project: editor_context.project(),
        io: editor_context.io.clone(),
        doc_context: &doc_context,
    };

    for component in &mut editor_context.components {
        component.draw(&mut egui_context, &component_ctx);
    }
}