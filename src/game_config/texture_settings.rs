use serde::{Deserialize, Serialize};

use super::PackageFormatConfig;

#[derive(Deserialize, Serialize, Debug)]
pub struct TextureConfig {
    pub package: TexturePackageConfig,
    pub format: PackageFormatConfig<TextureFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
}

impl Default for TextureConfig {
    fn default() -> Self {
        TextureConfig {
            package: TexturePackageConfig::Directory {
                root: "textures".to_string(),
            },
            format: PackageFormatConfig::<TextureFormat>::Extensions {
                extensions: vec!["png".to_string(), "jpg".to_string(), "jpeg".to_string()],
                format: TextureFormat::Image,
            },
            palette: None,
            attribute: Some("_tb_textures".to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum TexturePackageConfig {
    File { format: PackageFormatConfig<TexturePackageFormat> },
    Directory { root: String },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TexturePackageFormat {
    Wad2,
    Wad3,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TextureFormat {
    IdMip,
    HlMip,
    DkMip,
    Wal,
    Image,
}
