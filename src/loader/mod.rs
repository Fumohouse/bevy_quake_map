use crate::{data::Brush as BrushData, parsing::parse_map};
use anyhow::Result as AResult;
use bevy::{
    asset::{LoadContext, LoadedAsset},
    prelude::*,
    render::{
        render_resource::{AddressMode, SamplerDescriptor},
        texture::CompressedImageFormats,
    },
};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use glam::Vec3Swizzles;
use nom::error::Error as NomError;
use std::{collections::HashMap, str::Utf8Error, sync::Arc};
use thiserror::Error;

mod utils;

pub mod asset;
use asset::*;

const SCALE: f32 = 1.0 / 16.0;
const EMPTY_TEX: &str = "__TB_empty";

#[derive(Default)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Brush>().add_system(spawn_map_colliders);
    }
}

pub fn spawn_map_colliders(
    mut commands: Commands,
    query: Query<(Entity, &Brush), Without<Collider>>,
) {
    for (entity, brush) in query.iter() {
        commands
            .entity(entity)
            .insert(Collider::convex_hull(&brush.all_vertices).unwrap());
    }
}

#[async_trait]
pub trait MapAssetProvider: Send + Sync {
    async fn load_default_texture<'a>(
        &self,
        load_context: &'a mut LoadContext,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image>;

    async fn load_missing_texture<'a>(
        &self,
        load_context: &'a mut LoadContext,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image> {
        self.load_default_texture(load_context, supported_compressed_formats)
            .await
    }

    /// Load a texture from the map assets.
    /// Will only be called once per `tex_name`.
    async fn load_texture<'a>(
        &self,
        load_context: &'a mut LoadContext,
        supported_compressed_formats: CompressedImageFormats,
        tex_name: &str,
    ) -> Option<Image>;

    /// Create a material from the information provided,
    /// or use the default one if `None` is returned.
    /// Will only be called once per `tex_name`.
    async fn get_material<'a>(
        &self,
        _tex_name: &str,
        _load_context: &'a mut LoadContext,
        _default_tex: &Image,
    ) -> Option<StandardMaterial> {
        None
    }
}

#[derive(Error, Debug)]
pub enum MapError {
    #[error("can't load the default texture: {error}")]
    DefaultTextureLoadFailed { error: anyhow::Error },
    #[error("can't load the missing texture: {error}")]
    MissingTextureLoadFailed { error: anyhow::Error },
    #[error("not a valid utf-8 string")]
    Utf8Error(#[from] Utf8Error),
    #[error("failed to parse map")]
    ParseError,
}

async fn load_texture<'a, 'b>(
    tex_name: &'b str,
    load_context: &mut LoadContext<'_>,
    asset_provider: &Arc<dyn MapAssetProvider>,
    supported_compressed_formats: CompressedImageFormats,
    loaded_textures: &'a mut HashMap<&'b str, Image>,
) -> AResult<&'a Image, MapError> {
    if !loaded_textures.contains_key(tex_name) {
        let mut new_tex;

        if tex_name == EMPTY_TEX {
            new_tex = asset_provider
                .load_default_texture(load_context, supported_compressed_formats)
                .await
                .map_err(|error| MapError::DefaultTextureLoadFailed { error })?;
        } else {
            new_tex = match asset_provider
                .load_texture(load_context, supported_compressed_formats, tex_name)
                .await
            {
                Some(tex) => tex,
                None => asset_provider
                    .load_missing_texture(load_context, supported_compressed_formats)
                    .await
                    .map_err(|error| MapError::MissingTextureLoadFailed { error })?,
            };
        }

        new_tex.sampler_descriptor = SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..default()
        };

        loaded_textures.insert(tex_name, new_tex);
    }

    Ok(&loaded_textures[tex_name])
}

async fn load_material<'b>(
    tex_name: &'b str,
    load_context: &mut LoadContext<'_>,
    asset_provider: &Arc<dyn MapAssetProvider>,
    texture: &Image,
    loaded_materials: &mut HashMap<&'b str, Handle<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    if !loaded_materials.contains_key(tex_name) {
        let custom_material = asset_provider
            .get_material(tex_name, load_context, texture)
            .await;

        let material = custom_material.unwrap_or_else(|| {
            let tex_handle = load_context
                .set_labeled_asset(&tex_label(tex_name), LoadedAsset::new(texture.clone()));

            StandardMaterial {
                base_color_texture: Some(tex_handle),
                ..default()
            }
        });

        let material_handle =
            load_context.set_labeled_asset(&mat_label(tex_name), LoadedAsset::new(material));

        loaded_materials.insert(tex_name, material_handle);
    }

    loaded_materials[tex_name].clone()
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Brush {
    pub(crate) all_vertices: Vec<Vec3>,
}

async fn load_brush<'a, 'b>(
    entity_idx: usize,
    brush_idx: usize,
    brush: &'b BrushData,
    world: &mut World,
    load_context: &mut LoadContext<'_>,
    asset_provider: &Arc<dyn MapAssetProvider>,
    supported_compressed_formats: CompressedImageFormats,
    loaded_textures: &'a mut HashMap<&'b str, Image>,
    loaded_materials: &'a mut HashMap<&'b str, Handle<StandardMaterial>>,
) -> AResult<Entity, MapError> {
    let mut mesh_infos: HashMap<&str, _> = HashMap::new();
    let mut all_vertices = Vec::new();

    let faces = &brush.faces;

    // Collect all vertices from each brush face
    for i in 0..faces.len() {
        let face = &faces[i];
        let mut face_vertices = Vec::new();

        for j in 0..faces.len() {
            if i == j {
                continue;
            }

            for k in (j + 1)..faces.len() {
                if i == k || j == k {
                    continue;
                }

                if let Some(vertex) = face.intersect_faces(&faces[j], &faces[k]) {
                    if brush.contains(vertex) {
                        let vertex = vertex.as_vec3();

                        face_vertices.push(vertex);
                        all_vertices.push(vertex);
                    }
                }
            }
        }

        let u = face.u.axis.as_vec3();
        let v = face.v.axis.as_vec3();

        utils::wind(u, v, &mut face_vertices);

        let entry = mesh_infos
            .entry(&face.texture)
            .or_insert_with(|| BrushMeshInfo::default());

        entry.push_vertices(face, &face_vertices);
    }

    let centroid = all_vertices.iter().sum::<Vec3>() / (all_vertices.len() as f32);

    let mut ecs_meshes = Vec::new();

    for (tex_name, mesh_info) in mesh_infos {
        let texture = load_texture(
            tex_name,
            load_context,
            asset_provider,
            supported_compressed_formats,
            loaded_textures,
        )
        .await?;

        let mesh = mesh_info.to_mesh(centroid, texture.size());
        let mesh_handle = load_context.set_labeled_asset(
            &mesh_label(entity_idx, brush_idx, tex_name),
            LoadedAsset::new(mesh),
        );

        let material_handle = load_material(
            tex_name,
            load_context,
            asset_provider,
            texture,
            loaded_materials,
        )
        .await;

        let ecs_mesh = world
            .spawn()
            .insert_bundle(PbrBundle {
                mesh: mesh_handle,
                material: material_handle,
                ..default()
            })
            .id();

        ecs_meshes.push(ecs_mesh);
    }

    let all_vertices_transformed = all_vertices
        .iter()
        .map(|v| ((*v - centroid) * SCALE).yzx())
        .collect::<Vec<_>>();

    let ecs_brush = world
        .spawn()
        .insert_bundle(TransformBundle::from(Transform::from_translation(
            (centroid * SCALE).yzx(),
        )))
        .insert(Brush {
            all_vertices: all_vertices_transformed,
        })
        .insert(RigidBody::Fixed)
        .push_children(&ecs_meshes)
        .id();

    Ok(ecs_brush)
}

pub async fn load_map<'a>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'_>,
    supported_compressed_formats: CompressedImageFormats,
    asset_provider: Arc<dyn MapAssetProvider>,
) -> AResult<LoadedAsset<Scene>, MapError> {
    let map_text = std::str::from_utf8(bytes)?;
    let map = parse_map::<NomError<&str>>(map_text)
        .map_err(|_| MapError::ParseError)?
        .1;

    let mut loaded_textures = HashMap::new();
    let mut loaded_materials = HashMap::new();

    let mut world = World::new();

    let mut ecs_entities = Vec::new();

    for (entity_idx, entity) in map.entities.iter().enumerate() {
        let mut ecs_brushes = Vec::new();

        for (brush_idx, brush) in entity.brushes.iter().enumerate() {
            let entity = load_brush(
                entity_idx,
                brush_idx,
                brush,
                &mut world,
                load_context,
                &asset_provider,
                supported_compressed_formats,
                &mut loaded_textures,
                &mut loaded_materials,
            )
            .await?;

            ecs_brushes.push(entity);
        }

        let ecs_entity = world
            .spawn()
            .insert_bundle(TransformBundle::identity())
            .push_children(&ecs_brushes)
            .id();

        ecs_entities.push(ecs_entity);
    }

    world
        .spawn()
        .insert_bundle(TransformBundle::identity())
        .push_children(&ecs_entities);

    Ok(LoadedAsset::new(Scene::new(world)))
}

fn mesh_label(entity_idx: usize, brush_idx: usize, tex_name: &str) -> String {
    format!("Mesh_{}_{}_{}", entity_idx, brush_idx, tex_name)
}

fn tex_label(tex_name: &str) -> String {
    format!("Tex_{}", tex_name)
}

fn mat_label(tex_name: &str) -> String {
    format!("Mat_{}", tex_name)
}