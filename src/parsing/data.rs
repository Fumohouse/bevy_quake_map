use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(PartialEq, Debug)]
pub struct UvAxis {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub offset: f32,
}

#[derive(PartialEq, Debug)]
pub struct BrushFace {
    pub points: [Point; 3],
    pub texture: String,
    pub u: UvAxis,
    pub v: UvAxis,
    pub rotation: f32,
    pub x_scale: f32,
    pub y_scale: f32,
}

#[derive(PartialEq, Debug)]
pub struct Brush {
    pub faces: Vec<BrushFace>,
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
