use super::EditorDocumentItem;
use serde::{Deserialize, Serialize};

pub const SETTINGS_FILE: &str = "settings.ron";

#[derive(Serialize, Deserialize)]
pub struct GameSettings {
    pub name: String,
    pub description: String,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            name: "New Game".to_string(),
            description: Default::default(),
        }
    }
}

impl EditorDocumentItem for GameSettings {
    fn deserialize(
        serialized: &str,
        _doc_context: &super::DocumentIoContext,
    ) -> Result<Self, super::DocumentIoError> {
        Ok(ron::from_str::<GameSettings>(serialized)?)
    }

    fn serialize(
        &self,
        _doc_context: &super::DocumentIoContext,
    ) -> Result<String, super::DocumentIoError> {
        Ok(ron::to_string(self)?)
    }

    fn save_path(&self) -> String {
        SETTINGS_FILE.to_string()
    }

    fn name(&self) -> &str {
        "settings"
    }

    fn set_name(&mut self, _new_name: &str) {
        panic!("The settings cannot be renamed.")
    }
}
