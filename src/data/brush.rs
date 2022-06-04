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

#[cfg(test)]
mod tests {
    use crate::test_utils::get_brush;
    use glam::Vec3;

    #[test]
    fn test_brush_contains() {
        let brush = get_brush();

        assert!(brush.contains(Vec3::new(0.0, 0.0, 0.0)));
        assert!(brush.contains(Vec3::new(16.0, 16.0, 16.0)));
        assert!(!brush.contains(Vec3::new(16.0, 16.0, 16.0 + crate::EPSILON * 2.0)));
    }
}
