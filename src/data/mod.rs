use glam::Vec3;
use std::collections::HashMap;

mod brush;
pub use brush::*;

mod brush_face;
pub use brush_face::*;

#[derive(PartialEq, Debug)]
pub struct UvAxis {
    pub axis: Vec3,
    pub offset: f32,
}

#[derive(PartialEq, Debug)]
pub struct Entity {
    pub properties: HashMap<String, String>,
    pub brushes: Vec<Brush>,
}

#[derive(PartialEq, Debug)]
pub struct Map {
    pub entities: Vec<Entity>,
}
