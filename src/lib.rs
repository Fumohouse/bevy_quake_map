//! This crate references many online resources and the Qodot source
//! (https://github.com/QodotPlugin/libmap/blob/master/src/c/geo_generator.c).

pub const EPSILON: f32 = 0.001;
pub const EPSILON_64: f64 = EPSILON as f64;

extern crate bevy;
extern crate heron;
extern crate nom;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate async_trait;

pub mod data;
pub mod game_config;
pub mod parsing;

mod loader;
pub use loader::*;

mod test_utils;
