use anyhow::Result as AResult;
use bevy::{
    asset::LoadContext,
    pbr::StandardMaterial,
    prelude::{AssetServer, FromWorld, Image, World},
    render::{
        renderer::RenderDevice,
        texture::{CompressedImageFormats, ImageType},
    },
};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileTextureLoadError {
    #[error("texture collections undefined")]
    MissingTextureCollections,
    #[error("texture collections property is empty")]
    EmptyTextureCollections,
    #[error("failed to parse texture directory")]
    DirectoryParseFailed,
    #[error("failed to parse texture filename")]
    FilenameParseFailed,
    #[error("file not found")]
    FileNotFound,
    #[error("file has no extension")]
    NoExtension,
}

/// A relatively versatile and sensible default `MapAssetProvider`.
/// Uses the current `AssetIo` to load textures from the directory defined in the map's `worldspawn` entity.
/// Supports any file extension, as long as Bevy can load it.
pub struct FileAssetProvider {
    asset_server: AssetServer,
    default_texture_path: String,
}

impl FromWorld for FileAssetProvider {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();

        Self {
            asset_server,
            default_texture_path: "textures/default.png".to_string(),
        }
    }
}

impl FileAssetProvider {
    async fn try_load_texture_path(
        &self,
        path: &Path,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image> {
        let image_type = ImageType::Extension(
            path.extension()
                .ok_or(FileTextureLoadError::NoExtension)?
                .to_str()
                .unwrap(),
        );

        let buf = self.asset_server.asset_io().load_path(path).await?;

        Ok(Image::from_buffer(
            &buf,
            image_type,
            supported_compressed_formats,
            true,
        )?)
    }

    pub async fn try_load_texture(
        &self,
        load_context: &mut LoadContext<'_>,
        texture_collections: Option<&[&str]>,
        supported_compressed_formats: CompressedImageFormats,
        tex_name: &str,
    ) -> AResult<Image> {
        let collection_dir = *texture_collections
            .ok_or(FileTextureLoadError::MissingTextureCollections)?
            .get(0)
            .ok_or(FileTextureLoadError::EmptyTextureCollections)?;

        let mut texture_path = PathBuf::new();

        if let Some(path) = load_context.path().parent() {
            texture_path.push(path);
        }
        texture_path.push(collection_dir);
        texture_path.push(tex_name);

        let directory = texture_path
            .parent()
            .ok_or(FileTextureLoadError::DirectoryParseFailed)?;

        let filename_without_ext = texture_path
            .file_name()
            .ok_or(FileTextureLoadError::FilenameParseFailed)?;

        let file = self
            .asset_server
            .asset_io()
            .read_directory(directory)?
            .find(|p| p.as_path().file_stem().unwrap() == filename_without_ext)
            .ok_or(FileTextureLoadError::FileNotFound)?;

        self.try_load_texture_path(file.as_path(), supported_compressed_formats)
            .await
    }
}

#[async_trait]
impl MapAssetProvider for FileAssetProvider {
    async fn load_default_texture(
        &self,
        _load_context: &mut LoadContext,
        _texture_collections: Option<&[&str]>,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image> {
        self.try_load_texture_path(
            Path::new(&self.default_texture_path),
            supported_compressed_formats,
        )
        .await
    }

    async fn load_texture(
        &self,
        load_context: &mut LoadContext,
        texture_collections: Option<&[&str]>,
        supported_compressed_formats: CompressedImageFormats,
        tex_name: &str,
    ) -> Option<Image> {
        self.try_load_texture(
            load_context,
            texture_collections,
            supported_compressed_formats,
            tex_name,
        )
        .await
        .ok()
    }
}

pub fn get_supported_compressed_formats(world: &mut World) -> CompressedImageFormats {
    match world.get_resource::<RenderDevice>() {
        Some(render_device) => CompressedImageFormats::from_features(render_device.features()),
        None => CompressedImageFormats::all(),
    }
}

#[async_trait]
pub trait MapAssetProvider: Send + Sync {
    async fn load_default_texture(
        &self,
        load_context: &mut LoadContext,
        texture_collections: Option<&[&str]>,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image>;

    async fn load_missing_texture(
        &self,
        load_context: &mut LoadContext,
        texture_collections: Option<&[&str]>,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image> {
        self.load_default_texture(
            load_context,
            texture_collections,
            supported_compressed_formats,
        )
        .await
    }

    /// Load a texture from the map assets.
    /// Will only be called once per `tex_name`.
    async fn load_texture(
        &self,
        load_context: &mut LoadContext,
        texture_collections: Option<&[&str]>,
        supported_compressed_formats: CompressedImageFormats,
        tex_name: &str,
    ) -> Option<Image>;

    /// Create a material from the information provided,
    /// or use the default one if `None` is returned.
    /// Will only be called once per `tex_name`.
    async fn get_material(
        &self,
        _tex_name: &str,
        _load_context: &mut LoadContext,
        _default_tex: &Image,
    ) -> Option<StandardMaterial> {
        None
    }
}
