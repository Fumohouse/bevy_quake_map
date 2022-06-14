use glam::DVec3;
use std::collections::HashMap;

mod brush;
pub use brush::*;

mod brush_face;
pub use brush_face::*;

#[derive(PartialEq, Debug)]
pub struct UvAxis {
    pub axis: DVec3,
    pub offset: f64,
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

impl Map {
    pub fn worldspawn(&self) -> Option<&Entity> {
        self.entities.iter().find(|e| {
            if let Some("worldspawn") = e.properties.get("classname").map(|prop| prop.as_str()) {
                return true;
            }

            false
        })
    }
}
