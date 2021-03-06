use super::utils::{map_to_bevy_space3, map_to_bevy_space4};
use crate::map_data::BrushFace;
use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, PrimitiveTopology},
};
use glam::{Vec2, Vec3, Vec4};

/// Representation of one mesh in a Brush, which is associated with a particular material.
/// Coordinates are in .map space, and are not transformed until conversion to Mesh.
/// UVs are unscaled too, as textures are not loaded at this stage.
#[derive(Debug, Default)]
pub struct BrushMeshInfo {
    vertices: Vec<Vec3>,
    indices: Vec<usize>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    tangents: Vec<Vec4>,
}

impl BrushMeshInfo {
    /// Pushes a set of vertices, which should be defined by a face and already wound
    pub fn push_vertices(&mut self, brush_face: &BrushFace, vertices: &[Vec3]) {
        let u = &brush_face.u;
        let v = &brush_face.v;

        // Index of the first vertex in this set
        let begin_idx = self.vertices.len();

        for vertex in vertices {
            self.vertices.push(*vertex);
            self.normals.push(brush_face.normal.as_vec3());
            self.tangents.push(brush_face.tangent().as_vec4());

            // UV calculations can take place without swizzles or transformations
            let mut u_coord = u.axis.as_vec3().dot(*vertex);
            let mut v_coord = v.axis.as_vec3().dot(*vertex);

            // This scale does not affect the translation
            u_coord /= brush_face.x_scale;
            v_coord /= brush_face.y_scale;

            // Measured in pixels
            u_coord += u.offset as f32;
            v_coord += v.offset as f32;

            self.uvs.push(Vec2::new(u_coord, v_coord));
        }

        // Last vertex in this set
        let end_idx = self.vertices.len() - 1;

        // Perform fan triangulation on this set of vertices
        for i in (begin_idx + 1)..=(end_idx - 1) {
            self.indices.push(begin_idx);
            self.indices.push(i);
            self.indices.push(i + 1);
        }
    }

    pub fn to_mesh(&self, centroid: Vec3, tex_size: Vec2) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.vertices
                .iter()
                .map(|v| map_to_bevy_space3(&((*v - centroid) * super::SCALE)))
                .collect::<Vec<_>>(),
        );

        mesh.set_indices(Some(Indices::U32(
            self.indices
                .iter()
                .map(|i| *i as u32) // TODO: Smelly
                .collect::<Vec<_>>(),
        )));

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            self.normals
                .iter()
                .map(map_to_bevy_space3)
                .collect::<Vec<_>>(),
        );

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_TANGENT,
            self.tangents
                .iter()
                .map(map_to_bevy_space4)
                .collect::<Vec<_>>(),
        );

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            self.uvs
                .iter()
                .map(|uv| (*uv / tex_size).to_array())
                .collect::<Vec<_>>(),
        );

        mesh
    }
}
