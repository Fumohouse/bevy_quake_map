use crate::{
    document::{
        entity::{EntityDefinition, ENTITIES_DIR},
        DocumentIoContext, DocumentIoError, EditorDocument,
    },
    io::EditorIo,
};
use bevy::prelude::*;
use std::path::Path;

#[derive(Default)]
pub struct EditorProject {
    pub entities: Vec<EditorDocument<EntityDefinition>>,
}

fn search_directory(io: &dyn EditorIo, path: &Path, cb: &mut dyn FnMut(Vec<u8>)) {
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
        let mut project = Self::default();

        search_directory(io, Path::new(ENTITIES_DIR), &mut |contents| {
            EditorDocument::<EntityDefinition>::load_buf(&contents, &doc_context)
                .map(|doc| project.entities.push(doc))
                .unwrap_or_else(|err| {
                    warn!("Failed to parse entity definition: {}", err);
                });
        });

        project
    }

    pub fn save(
        &self,
        io: &dyn EditorIo,
        doc_context: DocumentIoContext,
    ) -> Result<(), DocumentIoError> {
        for doc in &self.entities {
            doc.save(io, &doc_context).unwrap_or_else(|err| {
                warn!("Failed to save document: {}", err);
            });
        }

        Ok(())
    }
}
