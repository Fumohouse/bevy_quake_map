#[macro_use]
extern crate async_trait;

use anyhow::Result as AResult;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext},
    prelude::*,
    render::{
        renderer::RenderDevice,
        texture::{CompressedImageFormats, ImageType},
    },
};
use bevy_flycam::PlayerPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_quake_map::{asset::MapAsset, load_map, spawner::MapSpawner, MapAssetProvider, MapPlugin};
use heron::{CollisionShape, Gravity, PhysicsPlugin, RigidBody};
use std::sync::Arc;

struct FileAssetProvider;

#[async_trait]
impl MapAssetProvider for FileAssetProvider {
    async fn load_default_texture<'a>(
        &self,
        load_context: &'a mut LoadContext,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image> {
        let buf = load_context
            .read_asset_bytes("textures/default.png")
            .await?;

        Ok(Image::from_buffer(
            &buf,
            ImageType::MimeType("image/png"),
            supported_compressed_formats,
            true,
        )?)
    }

    async fn load_texture<'a>(
        &self,
        load_context: &'a mut LoadContext,
        supported_compressed_formats: CompressedImageFormats,
        tex_name: &str,
    ) -> Option<Image> {
        let buf = load_context
            .read_asset_bytes(format!("textures/{}.png", tex_name))
            .await
            .ok()?;

        Some(
            Image::from_buffer(
                &buf,
                ImageType::MimeType("image/png"),
                supported_compressed_formats,
                true,
            )
            .ok()?,
        )
    }
}

struct MapLoader {
    asset_provider: Arc<dyn MapAssetProvider>,
    supported_compressed_formats: CompressedImageFormats,
}

impl FromWorld for MapLoader {
    fn from_world(world: &mut World) -> Self {
        let supported_compressed_formats = match world.get_resource::<RenderDevice>() {
            Some(render_device) => CompressedImageFormats::from_features(render_device.features()),
            None => CompressedImageFormats::all(),
        };

        Self {
            asset_provider: Arc::new(FileAssetProvider),
            supported_compressed_formats,
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
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, -10.0, 0.0)))
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(PlayerPlugin)
        .add_plugin(MapPlugin)
        .init_asset_loader::<MapLoader>()
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_spawner: ResMut<MapSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    asset_server.watch_for_changes().unwrap();

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            transform: Transform::from_xyz(0.0, 20.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.5, 0.5, 0.5),
            border_radius: None,
        });

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

    let handle = asset_server.load::<MapAsset, _>("test.map");
    map_spawner.spawn(handle);
}
