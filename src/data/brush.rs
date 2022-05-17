use super::BrushFace;
use glam::Vec3;

#[derive(PartialEq, Debug)]
pub struct Brush {
    pub faces: Vec<BrushFace>,
}

impl Brush {
    pub fn contains(&self, point: Vec3) -> bool {
        // This works because brushes must be convex
        for face in &self.faces {
            if face.normal.dot(point) - face.origin_dist > crate::EPSILON {
                return false;
            }
        }

        return true;
    }
}
