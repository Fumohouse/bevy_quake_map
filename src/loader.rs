use glam::{Vec2, Vec3};
use std::cmp::Ordering;

fn project(u: Vec3, v: Vec3, point: Vec3) -> Vec2 {
    Vec2::new(u.dot(point), v.dot(point))
}

/// Sorts an arbitrary number of vertices in counter-clockwise order
/// assuming they form a convex shape
fn wind(u: Vec3, v: Vec3, vertices: &[Vec3]) -> Vec<Vec3> {
    let mut copy = vertices.to_vec();
    let center = vertices.iter().sum::<Vec3>() / (vertices.len() as f32);
    let center_proj = project(u, v, center);

    copy.sort_by(|a, b| {
        let a_proj = project(u, v, *a);
        let b_proj = project(u, v, *b);

        let a_off = a_proj - center_proj;
        let b_off = b_proj - center_proj;

        // https://stackoverflow.com/a/6989383
        // | a.x b.x |
        // | a.y b.y |
        let det = a_off.x * b_off.y - b_off.x * a_off.y;

        if det < 0.0 {
            return Ordering::Greater;
        } else if det > 0.0 {
            return Ordering::Less;
        }

        panic!("this set of points cannot be wound");
    });

    copy
}

#[cfg(test)]
mod tests {
    use super::wind;
    use crate::test_utils::get_brush;
    use glam::Vec3;
    use std::f32::consts::TAU;

    #[test]
    fn test_brush_contains() {
        let brush = get_brush();

        assert!(brush.contains(Vec3::new(0.0, 0.0, 0.0)));
        assert!(brush.contains(Vec3::new(16.0, 16.0, 16.0)));
        assert!(!brush.contains(Vec3::new(16.0, 16.0, 16.0 + crate::EPSILON * 2.0)));
    }

    fn polygon(rad: f32, sides: usize) -> (Vec<Vec3>, Vec3, Vec3) {
        let mut points = Vec::new();

        for i in 0..sides {
            let theta = (TAU / (sides as f32)) * (i as f32);

            points.push(rad * Vec3::new(theta.cos(), theta.sin(), 0.0));
        }

        (points, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0))
    }

    #[test]
    fn test_vertex_wind() {
        const RAD: f32 = 5.0;

        let (vertices, u, v) = polygon(RAD, 5);

        let scrambled = vec![
            vertices[1],
            vertices[0],
            vertices[2],
            vertices[4],
            vertices[3],
        ];

        let wound = wind(u, v, &scrambled);
        assert_eq!(wound, vertices);
    }
}