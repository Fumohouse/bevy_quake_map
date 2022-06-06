use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct FaceAttributes {
    #[serde(rename = "surfaceflags")]
    pub surface_flags: Vec<FaceFlag>,
    #[serde(rename = "contentflags")]
    pub content_flags: Vec<FaceFlag>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum FaceFlag {
    Unused {
        unused: bool,
    },
    Used {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        defaults: Option<FaceAttributeDefaults>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FaceAttributeDefaults {
    pub offset: Option<[f32; 2]>,
    pub scale: Option<[f32; 2]>,
    pub rotation: Option<f32>,
    #[serde(rename = "surfaceValue")]
    pub surface_value: Option<f32>,
    #[serde(rename = "surfaceFlags")]
    pub surface_flags: Option<Vec<String>>,
    #[serde(rename = "surfaceContents")]
    pub surface_contents: Option<Vec<String>>,
    pub color: Option<String>,
}
