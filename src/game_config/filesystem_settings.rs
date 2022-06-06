use serde::{Deserialize, Serialize};

use super::PackageFormatSettings;

#[derive(Deserialize, Serialize, Debug)]
pub struct FilesystemSettings {
    #[serde(rename = "searchpath")]
    pub search_path: String,
    #[serde(rename = "packageformat")]
    pub package_format: PackageFormatSettings<PackageFormat>,
}

impl Default for FilesystemSettings {
    fn default() -> Self {
        FilesystemSettings {
            search_path: ".".to_string(),
            package_format: PackageFormatSettings::<PackageFormat>::Extension {
                extension: "pak".to_string(),
                format: PackageFormat::IdPak,
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PackageFormat {
    IdPak,
    DkPak,
    Zip,
}
