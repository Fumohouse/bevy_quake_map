use crate::{
    document::{
        entity::{EntityDefinition, ENTITIES_DIR},
        game_settings::{GameSettings, SETTINGS_FILE},
        DocumentCollection, DocumentIoContext, DocumentIoError, DocumentState, EditorDocument,
    },
    io::EditorIo,
};
use bevy::prelude::*;
use std::path::Path;

pub struct EditorProject {
    pub settings: EditorDocument<GameSettings>,
    pub entities: DocumentCollection<EntityDefinition>,
}

fn search_directory(io: &dyn EditorIo, path: &Path, mut cb: impl FnMut(Vec<u8>)) {
    if let Ok(files) = io.read_directory(path) {
        for file in files {
            if let Ok(contents) = io.read_file(&file) {
                cb(contents);
            }
        }
    }
}

impl EditorProject {
    pub fn load(io: &dyn EditorIo, doc_context: DocumentIoContext) -> Self {
        let settings = io
            .read_file(Path::new(SETTINGS_FILE))
            .map(|buf| {
                EditorDocument::<GameSettings>::load_buf(&buf, &doc_context).unwrap_or_else(|err| {
                    panic!("Failed to parse game settings: {}", err);
                })
            })
            .unwrap_or_else(|_| EditorDocument::new(GameSettings::default(), DocumentState::New));

        let mut entities = DocumentCollection::<EntityDefinition>::default();

        search_directory(io, Path::new(ENTITIES_DIR), |contents| {
            EditorDocument::<EntityDefinition>::load_buf(&contents, &doc_context)
                .map(|doc| entities.insert(doc))
                .unwrap_or_else(|err| {
                    warn!("Failed to parse entity definition: {}", err);
                });
        });

        Self { settings, entities }
    }

    pub fn save(
        &self,
        io: &dyn EditorIo,
        doc_context: DocumentIoContext,
    ) -> Result<(), DocumentIoError> {
        self.settings.save(io, &doc_context).unwrap_or_else(|err| {
            warn!("Failed to save settings: {}", err);
        });

        for doc in self.entities.values() {
            doc.save(io, &doc_context).unwrap_or_else(|err| {
                warn!("Failed to save entity definition: {}", err);
            });
        }

        Ok(())
    }
}
