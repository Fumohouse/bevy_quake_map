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
    use glam::Vec3;

    use crate::{parsing::parse_map, test_data};

    #[test]
    fn test_brush_contains() {
        let map = parse_map::<()>(test_data::TEST_MAP)
            .expect("failed to parse")
            .1;

        let brush = map
            .entities
            .get(0)
            .expect("no entity")
            .brushes
            .get(0)
            .expect("no brush");

        assert!(brush.contains(Vec3::new(0.0, 0.0, 0.0)));
        assert!(brush.contains(Vec3::new(16.0, 16.0, 16.0)));
        assert!(!brush.contains(Vec3::new(16.0, 16.0, 16.0 + crate::EPSILON * 2.0)));
    }
}
