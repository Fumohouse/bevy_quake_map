use super::UvAxis;
use glam::{Vec3, Vec4};

#[derive(PartialEq, Debug)]
pub struct BrushFace {
    /// Point layout:
    /// 0----1
    /// |
    /// |
    /// 2
    /// The normal points out of the screen.
    /// (https://www.gamers.org/dEngine/quake/QDP/qmapspec.html)
    pub points: [Vec3; 3],
    pub texture: String,
    pub u: UvAxis,
    pub v: UvAxis,
    pub rotation: f32,
    pub x_scale: f32,
    pub y_scale: f32,

    pub normal: Vec3,
    pub origin_dist: f32,
}

impl BrushFace {
    pub fn new(
        points: [Vec3; 3],
        texture: String,
        u: UvAxis,
        v: UvAxis,
        rotation: f32,
        x_scale: f32,
        y_scale: f32,
    ) -> BrushFace {
        // https://stackoverflow.com/a/1966605
        // (except reversed since points are CW not CCW)
        let normal = (points[2] - points[0]).cross(points[1] - points[0]);
        let origin_dist = normal.dot(points[0]);

        BrushFace {
            points,
            texture,
            u,
            v,
            rotation,
            x_scale,
            y_scale,
            normal,
            origin_dist,
        }
    }

    pub fn intersect_faces(&self, f2: &BrushFace, f3: &BrushFace) -> Option<Vec3> {
        // https://math.stackexchange.com/a/3734749 (IDK how this works)
        let determinant = self.normal.dot(f2.normal.cross(f3.normal));

        if determinant.abs() < crate::EPSILON {
            return None;
        }

        // https://mathworld.wolfram.com/Plane-PlaneIntersection.html
        Some(
            (self.origin_dist * (f2.normal.cross(f3.normal))
                + f2.origin_dist * (f3.normal.cross(self.normal))
                + f3.origin_dist * (self.normal.cross(f2.normal)))
                / determinant,
        )
    }

    pub fn tangent(&self) -> Vec4 {
        // u: tangent, v: bitangent
        // cross(normal, u) * w = v
        // -> w = sign(dot(cross(normal, u), v))
        let u = self.u.axis;
        let w = (self.normal.cross(u).dot(self.v.axis)).signum();

        Vec4::new(u.x, u.y, u.z, w)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::get_brush;
    use glam::Vec3;

    #[test]
    fn test_intersect_faces() {
        let brush = get_brush();

        let intersect = brush.faces[0]
            .intersect_faces(&brush.faces[1], &brush.faces[2])
            .expect("failed to find intersect");

        assert!(intersect.abs_diff_eq(Vec3::new(-16.0, -16.0, -16.0), crate::EPSILON))
    }

    #[test]
    fn test_tangent() {
        let brush = get_brush();
        let face = brush.faces.get(0).expect("no face");

        let tangent = face.tangent();

        assert!(
            (face.normal.cross(face.u.axis) * tangent.w).abs_diff_eq(face.v.axis, crate::EPSILON)
        );
    }
}
