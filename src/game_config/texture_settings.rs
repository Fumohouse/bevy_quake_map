use super::PackageFormatSettings;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct TextureSettings {
    pub package: TexturePackageType,
    pub format: PackageFormatSettings<TextureFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
}

impl Default for TextureSettings {
    fn default() -> Self {
        TextureSettings {
            package: TexturePackageType::Directory {
                root: "textures".to_string(),
            },
            format: PackageFormatSettings::<TextureFormat>::Extensions {
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
pub enum TexturePackageType {
    File {
        format: PackageFormatSettings<FilePackageFormat>,
    },
    Directory {
        root: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FilePackageFormat {
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
