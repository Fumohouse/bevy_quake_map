use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Tags {
    pub brush: Vec<Tag>,
    #[serde(rename = "brushface")]
    pub brush_face: Vec<Tag>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribs: Option<Vec<TagAttribute>>,
    pub r#match: TagMatchType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TagAttribute {
    Transparent,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TagMatchType {
    ClassName,
    Texture,
    ContentFlag,
    SurfaceFlag,
    SurfaceParam,
}
