use crate::{
    document::DocumentIoContext,
    io::{EditorIo, EditorIoError},
};

#[derive(Debug, Default)]
pub struct EditorProject {}

impl EditorProject {
    pub fn load(io: &dyn EditorIo, doc_context: DocumentIoContext) -> Self {
        let project = Self::default();

        // TODO

        project
    }

    pub fn save(&self, io: &dyn EditorIo) -> Result<(), EditorIoError> {
        todo!()
    }
}
