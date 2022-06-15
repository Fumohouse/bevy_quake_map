//! This crate references many online resources and the Qodot source
//! (https://github.com/QodotPlugin/libmap/blob/master/src/c/geo_generator.c).

pub const EPSILON: f32 = 0.001;
pub const EPSILON_64: f64 = EPSILON as f64;

#[macro_use]
extern crate async_trait;

pub mod map_data;
pub mod game_config;
pub mod parsing;

mod loader;
pub use loader::*;

mod test_utils;
