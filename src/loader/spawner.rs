//! Based on bevy_scene scene_spawner

use super::asset::MapAsset;
use bevy::prelude::*;
use heron::RigidBody;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MapSpawnError {
    #[error("the map is not loaded")]
    MapNotLoaded,
}

#[derive(Default)]
pub struct MapSpawner {
    queued_maps: Vec<Handle<MapAsset>>,
}

impl MapSpawner {
    pub fn spawn(&mut self, handle: Handle<MapAsset>) {
        self.queued_maps.push(handle);
    }

    fn spawn_handle(world: &mut World, handle: Handle<MapAsset>) -> Result<(), MapSpawnError> {
        world.resource_scope(|world, assets: Mut<Assets<MapAsset>>| {
            let map = assets
                .get(handle)
                .ok_or_else(|| MapSpawnError::MapNotLoaded)?;

            for entity in &map.entities {
                let mut ecs_entity = world.spawn();

                ecs_entity.with_children(|parent| {
                    for brush in &entity.brushes {
                        let mut ecs_brush = parent.spawn_bundle(TransformBundle::from_transform(
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

            Ok(())
        })
    }

    fn spawn_queued(&mut self, world: &mut World) -> Result<(), MapSpawnError> {
        let queued = std::mem::take(&mut self.queued_maps);

        for handle in queued {
            match Self::spawn_handle(world, handle.clone()) {
                Ok(_) => {}
                Err(MapSpawnError::MapNotLoaded) => self.spawn(handle),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn system(world: &mut World) {
        world.resource_scope(|world, mut map_spawner: Mut<MapSpawner>| {
            map_spawner.spawn_queued(world).unwrap_or_else(|e| {
                panic!("map spawn failed: {}", e);
            });
        });
    }
}
