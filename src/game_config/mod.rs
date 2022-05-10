//! Structs and enums for TrenchBroom game configuration files
//! https://trenchbroom.github.io/manual/latest/#game_configuration_files
//! Names of some structs are borrowed from TrenchBroom source, which is GPLv3.

use serde::{Deserialize, Serialize};

mod entity_settings;
mod face_attribs;
mod filesystem_settings;
mod tags;
mod texture_settings;

pub use entity_settings::*;
pub use face_attribs::*;
pub use filesystem_settings::*;
pub use tags::*;
pub use texture_settings::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct GameConfig {
    pub version: u32,
    pub name: String,
    #[serde(rename = "fileformats")]
    pub file_formats: Vec<MapFormatConfig>,
    pub filesystem: FileSystemConfig,
    pub textures: TextureConfig,
    pub entities: EntityConfig,
    pub tags: Tags,
    #[serde(rename = "faceattribs")]
    pub face_attribs: FaceAttribsConfig,
    #[serde(rename = "softMapBounds", skip_serializing_if = "Option::is_none")]
    pub soft_map_bounds: Option<String>,
    #[serde(rename = "compilationTools", skip_serializing_if = "Option::is_none")]
    pub compilation_tools: Option<Vec<CompilationTool>>,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            version: 4,
            name: "A Quake Map".to_string(),
            file_formats: vec![MapFormatConfig {
                format: MapFormat::Valve,
                initial_map: Some("initial_valve.map".to_string()),
            }],
            filesystem: FileSystemConfig::default(),
            textures: TextureConfig::default(),
            entities: EntityConfig {
                definitions: Vec::new(),
                default_color: "0.6 0.6 0.6 1.0".to_string(),
                model_formats: vec![EntityModelFormat::Obj],
            },
            tags: Tags {
                brush: Vec::new(),
                brush_face: Vec::new(),
            },
            face_attribs: FaceAttribsConfig {
                surface_flags: Vec::new(),
                content_flags: Vec::new(),
            },
            soft_map_bounds: None,
            compilation_tools: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum PackageFormatConfig<T> {
    Extension { extension: String, format: T },
    Extensions { extensions: Vec<String>, format: T },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MapFormatConfig {
    pub format: MapFormat,
    #[serde(rename = "initialmap", skip_serializing_if = "Option::is_none")]
    pub initial_map: Option<String>,
}

// Some are undocumented
// https://github.com/TrenchBroom/TrenchBroom/blob/master/common/src/Model/MapFormat.cpp#L28-L50
#[derive(Deserialize, Serialize, Debug)]
pub enum MapFormat {
    Standard,
    Valve,
    Quake2,
    #[serde(rename = "Quake2 (Valve)")]
    Quake2Valve,
    Quake3,
    #[serde(rename = "Quake3 (legacy)")]
    Quake3Legacy,
    #[serde(rename = "Quake3 (Valve)")]
    Quake3Valve,
    Hexen2,
    Daikatana,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CompilationTool {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use crate::game_config::GameConfig;

    #[test]
    fn test_deserialize() {
        // The test data is generated by Qodot, and slightly altered for sanity.
        serde_json::from_str::<GameConfig>(r#"
{
	"version": 3,
	"name": "Fumohouse",
	"icon": "Icon.png",
	"fileformats": [
		{ "format": "Standard", "initialmap": "initial_standard.map" },
		{ "format": "Valve", "initialmap": "initial_valve.map" },
		{ "format": "Quake2", "initialmap": "initial_quake2.map" },
		{ "format": "Quake3" },
		{ "format": "Quake3 (legacy)" },
		{ "format": "Hexen2" },
		{ "format": "Daikatana" }
	],
	"filesystem": {
		"searchpath": ".",
		"packageformat": { "extension": "pak", "format": "idpak" }
	},
	"textures": {
		"package": { "type": "directory", "root": "textures" },
		"format": { "extensions": ["bmp", "exr", "hdr", "jpeg", "jpg", "png", "tga", "webp"], "format": "image" },
		"attribute": "_tb_textures"
	},
	"entities": {
		"definitions": [ "Qodot.fgd", "Fumohouse.fgd" ],
		"defaultcolor": "0.6 0.6 0.6 1.0",
		"modelformats": [ "mdl", "md2", "md3", "bsp", "dkm", "obj_neverball" ]
	},
	"tags": {
		"brush": [
			{
				"name": "Trigger",
				"attribs": [ "transparent" ],
				"match": "classname",
				"pattern": "trigger*",
				"texture": "trigger"
			},{
				"name": "Detail",
				"attribs": [  ],
				"match": "classname",
				"pattern": "detail*"
			}
		],
		"brushface": [
			{
				"name": "Clip",
				"attribs": [ "transparent" ],
				"match": "texture",
				"pattern": "clip"
			},{
				"name": "Skip",
				"attribs": [ "transparent" ],
				"match": "texture",
				"pattern": "skip"
			},{
				"name": "Window",
				"attribs": [ "transparent" ],
				"match": "texture",
				"pattern": "window",
				"texture": "window"
			}
		]
	},
	"faceattribs": {
		"surfaceflags": [

		],
		"contentflags": [

		]
	}
}
        "#).expect("Deserialization failed");
    }
}
