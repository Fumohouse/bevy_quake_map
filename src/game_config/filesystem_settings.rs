use serde::{Deserialize, Serialize};

use super::PackageFormatConfig;

#[derive(Deserialize, Serialize, Debug)]
pub struct FileSystemConfig {
    #[serde(rename = "searchpath")]
    pub search_path: String,
    #[serde(rename = "packageformat")]
    pub package_format: PackageFormatConfig<PackageFormat>,
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        FileSystemConfig {
            search_path: ".".to_string(),
            package_format: PackageFormatConfig::<PackageFormat>::Extension {
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
