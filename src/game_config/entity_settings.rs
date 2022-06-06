use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct EntitySettings {
    pub definitions: Vec<String>,
    #[serde(rename = "defaultcolor")]
    pub default_color: String,
    #[serde(rename = "modelformats")]
    pub model_formats: Vec<ModelFormat>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ModelFormat {
    Mdl,
    Md2,
    Md3,
    Bsp,
    Dkm,
    #[serde(rename = "obj_neverball")]
    Obj,
}
