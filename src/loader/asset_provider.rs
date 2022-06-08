use anyhow::Result as AResult;
use bevy::{
    asset::LoadContext,
    pbr::StandardMaterial,
    prelude::{Image, World},
    render::{renderer::RenderDevice, texture::CompressedImageFormats},
};

pub fn get_supported_compressed_formats(world: &mut World) -> CompressedImageFormats {
    match world.get_resource::<RenderDevice>() {
        Some(render_device) => CompressedImageFormats::from_features(render_device.features()),
        None => CompressedImageFormats::all(),
    }
}

#[async_trait]
pub trait MapAssetProvider: Send + Sync {
    async fn load_default_texture<'a>(
        &self,
        load_context: &'a mut LoadContext,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image>;

    async fn load_missing_texture<'a>(
        &self,
        load_context: &'a mut LoadContext,
        supported_compressed_formats: CompressedImageFormats,
    ) -> AResult<Image> {
        self.load_default_texture(load_context, supported_compressed_formats)
            .await
    }

    /// Load a texture from the map assets.
    /// Will only be called once per `tex_name`.
    async fn load_texture<'a>(
        &self,
        load_context: &'a mut LoadContext,
        supported_compressed_formats: CompressedImageFormats,
        tex_name: &str,
    ) -> Option<Image>;

    /// Create a material from the information provided,
    /// or use the default one if `None` is returned.
    /// Will only be called once per `tex_name`.
    async fn get_material<'a>(
        &self,
        _tex_name: &str,
        _load_context: &'a mut LoadContext,
        _default_tex: &Image,
    ) -> Option<StandardMaterial> {
        None
    }
}
