use glam::{Vec2, Vec3, Vec4};
use heron::rapier_plugin::nalgebra::{OPoint, Const};
use std::cmp::Ordering;

pub fn map_to_bevy_space3(v: &Vec3) -> [f32; 3] {
    [v.y, v.z, v.x]
}

pub fn map_to_rapier(v: &Vec3) -> OPoint<f32, Const<3>> {
    OPoint::<f32, Const<3>>::new(v.y, v.z, v.x)
}

pub fn map_to_bevy_space4(v: &Vec4) -> [f32; 4] {
    [v.y, v.z, v.x, v.w]
}

// Projects `point` onto a plane with axes `u` and `v`
fn project(u: Vec3, v: Vec3, point: Vec3) -> Vec2 {
    Vec2::new(u.dot(point), v.dot(point))
}

// TODO: Testing possible? Order is not generally guaranteed
/// Sorts an arbitrary number of vertices in clockwise order
/// assuming they form a convex shape (https://stackoverflow.com/a/6989383)
pub fn wind(u: Vec3, v: Vec3, vertices: &mut Vec<Vec3>) {
    let center = vertices.iter().sum::<Vec3>() / (vertices.len() as f32);
    let center_proj = project(u, v, center);

    vertices.sort_by(|a, b| {
        let a_proj = project(u, v, *a);
        let b_proj = project(u, v, *b);

        let a_off = a_proj - center_proj;
        let b_off = b_proj - center_proj;

        // Handle edge cases
        if a_off.x >= 0.0 && b_off.x < 0.0 {
            return Ordering::Less;
        }

        if a_off.x < 0.0 && b_off.x >= 0.0 {
            return Ordering::Greater;
        }

        if a_off.x == 0.0 && b_off.x == 0.0 {
            if a_off.y >= 0.0 || b_off.y >= 0.0 {
                if a_proj.y > b_proj.y {
                    return Ordering::Less;
                }

                return Ordering::Greater;
            }

            if b_proj.y > a_proj.y {
                return Ordering::Less;
            }

            return Ordering::Greater;
        }

        // | a.x b.x |
        // | a.y b.y |
        let det = a_off.x * b_off.y - b_off.x * a_off.y;

        if det < 0.0 {
            return Ordering::Less;
        } else if det > 0.0 {
            return Ordering::Greater;
        }

        panic!("this set of points cannot be wound");
    });
}
