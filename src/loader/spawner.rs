//! Based on bevy_scene scene_spawner

use super::asset::MapAsset;
use bevy::{
    ecs::event::{Events, ManualEventReader},
    hierarchy::{BuildWorldChildren, DespawnRecursiveExt},
    pbr::PbrBundle,
    prelude::{default, AssetEvent, Assets, Entity, Handle, Mut, Transform, World},
    transform::TransformBundle,
};
use heron::RigidBody;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MapSpawnError {
    #[error("the map is not loaded")]
    MapNotLoaded,
}

#[derive(Default)]
pub struct MapSpawner {
    queued_maps: Vec<Handle<MapAsset>>,
    event_reader: ManualEventReader<AssetEvent<MapAsset>>,
    entities: HashMap<Handle<MapAsset>, Entity>,
}

impl MapSpawner {
    pub fn spawn(&mut self, handle: Handle<MapAsset>) {
        self.queued_maps.push(handle);
    }

    fn spawn_handle(
        &mut self,
        world: &mut World,
        handle: Handle<MapAsset>,
    ) -> Result<(), MapSpawnError> {
        world.resource_scope(|world, assets: Mut<Assets<MapAsset>>| {
            let map = assets
                .get(handle.clone_weak())
                .ok_or_else(|| MapSpawnError::MapNotLoaded)?;

            let mut map_entity = world.spawn();
            map_entity.insert_bundle(TransformBundle::identity());

            map_entity.with_children(|parent| {
                let mut ecs_entity = parent.spawn_bundle(TransformBundle::identity());

                for entity in &map.entities {
                    ecs_entity.with_children(|parent| {
                        for brush in &entity.brushes {
                            let mut ecs_brush =
                                parent.spawn_bundle(TransformBundle::from_transform(
                                    Transform::from_translation(brush.position),
                                ));

                            ecs_brush
                                .insert(RigidBody::Static)
                                .insert(brush.collider.clone());

                            ecs_brush.with_children(|parent| {
                                for (_tex_name, (mesh_handle, mat_handle)) in &brush.meshes {
                                    parent.spawn_bundle(PbrBundle {
                                        mesh: mesh_handle.clone(),
                                        material: mat_handle.clone(),
                                        ..default()
                                    });
                                }
                            });
                        }
                    });
                }
            });

            self.entities.insert(handle.clone_weak(), map_entity.id());

            Ok(())
        })
    }

    fn spawn_queued(&mut self, world: &mut World) -> Result<(), MapSpawnError> {
        let queued = std::mem::take(&mut self.queued_maps);

        for handle in queued {
            match self.spawn_handle(world, handle.clone()) {
                Ok(_) => {}
                Err(MapSpawnError::MapNotLoaded) => self.spawn(handle),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn system(world: &mut World) {
        world.resource_scope(|world, mut map_spawner: Mut<MapSpawner>| {
            let map_asset_events = world.resource::<Events<AssetEvent<MapAsset>>>();

            let mut to_respawn = Vec::new();

            for event in map_spawner.event_reader.iter(map_asset_events) {
                if let AssetEvent::Modified { handle } = event {
                    to_respawn.push(handle.clone_weak());
                }
            }

            for handle in to_respawn {
                if let Some(entity) = map_spawner.entities.remove(&handle) {
                    world.entity_mut(entity).despawn_recursive();
                }

                map_spawner.queued_maps.push(handle);
            }

            map_spawner.spawn_queued(world).unwrap_or_else(|e| {
                panic!("map spawn failed: {}", e);
            });
        });
    }
}
