use anyhow::Result as AResult;
use bevy::{
    asset::{AssetLoader, BoxedFuture},
    prelude::*,
    render::texture::CompressedImageFormats,
};
use bevy_flycam::PlayerPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_quake_map::{
    get_supported_compressed_formats, load_map, FileAssetProvider, MapAssetProvider, MapPlugin,
};
use bevy_rapier3d::prelude::*;
use std::sync::Arc;

struct MapLoader {
    asset_provider: Arc<dyn MapAssetProvider>,
    supported_compressed_formats: CompressedImageFormats,
}

impl FromWorld for MapLoader {
    fn from_world(world: &mut World) -> Self {
        Self {
            asset_provider: Arc::new(FileAssetProvider::from_world(world)),
            supported_compressed_formats: get_supported_compressed_formats(world),
        }
    }
}

impl AssetLoader for MapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, AResult<()>> {
        Box::pin(async move {
            let map = load_map(
                bytes,
                load_context,
                self.supported_compressed_formats,
                self.asset_provider.clone(),
            )
            .await?;

            load_context.set_default_asset(map);

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(PlayerPlugin)
        .add_plugin(MapPlugin)
        .init_asset_loader::<MapLoader>()
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>) {
    asset_server.watch_for_changes().unwrap();

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            transform: Transform::from_xyz(0.0, 20.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5));

    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let handle = asset_server.load("test.map");
    commands.spawn_scene(handle);
}
