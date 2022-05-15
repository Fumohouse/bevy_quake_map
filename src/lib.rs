//! This crate references many online resources and the Qodot source
//! (https://github.com/QodotPlugin/libmap/blob/master/src/c/geo_generator.c).

pub const EPSILON: f32 = 0.001;

extern crate nom;
extern crate serde;
extern crate serde_json;

pub mod data;
pub mod game_config;
pub mod parsing;

mod test_utils;
