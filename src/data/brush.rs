use super::BrushFace;
use glam::DVec3;

#[derive(PartialEq, Debug)]
pub struct Brush {
    pub faces: Vec<BrushFace>,
}

impl Brush {
    pub fn contains(&self, point: DVec3) -> bool {
        // This works because brushes must be convex
        for face in &self.faces {
            if face.normal.dot(point) - face.origin_dist > crate::EPSILON_64 {
                return false;
            }
        }

        return true;
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::get_brush;
    use glam::DVec3;

    #[test]
    fn test_brush_contains() {
        let brush = get_brush();

        assert!(brush.contains(DVec3::new(0.0, 0.0, 0.0)));
        assert!(brush.contains(DVec3::new(16.0, 16.0, 16.0)));
        assert!(!brush.contains(DVec3::new(16.0, 16.0, 16.0 + crate::EPSILON_64 * 2.0)));
    }
}
