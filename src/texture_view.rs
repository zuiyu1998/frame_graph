use wgpu::{TextureAspect, TextureFormat, TextureUsages, TextureView, TextureViewDimension};

use crate::{
    ResourceRead, ResourceRef, ResourceView, ResourceWrite, TransientTexture, pass::PassContext,
};

pub type TransientTextureViewDescriptorRead = TransientTextureViewDescriptor<ResourceRead>;

pub type TransientTextureViewDescriptorWrite = TransientTextureViewDescriptor<ResourceWrite>;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct TextureViewDescriptor {
    pub label: Option<String>,
    pub format: Option<TextureFormat>,
    pub dimension: Option<TextureViewDimension>,
    pub usage: Option<TextureUsages>,
    pub aspect: TextureAspect,
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
}

impl TextureViewDescriptor {
    pub fn get_desc<'a>(&'a self) -> wgpu::TextureViewDescriptor<'a> {
        wgpu::TextureViewDescriptor {
            label: self.label.as_deref(),
            format: self.format,
            dimension: self.dimension,
            usage: self.usage,
            aspect: self.aspect,
            base_mip_level: self.base_mip_level,
            mip_level_count: self.mip_level_count,
            base_array_layer: self.base_array_layer,
            array_layer_count: self.array_layer_count,
        }
    }
}

pub struct TransientTextureViewDescriptor<ViewType> {
    pub texture: ResourceRef<TransientTexture, ViewType>,
    pub desc: TextureViewDescriptor,
}

impl<ViewType: ResourceView> TransientTextureViewDescriptor<ViewType> {
    pub fn create_gpu_texture_view(&self, context: &PassContext) -> TextureView {
        let resource = context.get_resource(&self.texture);
        resource.resource.create_view(&self.desc.get_desc())
    }
}

pub enum TransientTextureView {
    Read(TransientTextureViewDescriptorRead),
    Write(TransientTextureViewDescriptorWrite),
    Owned(TextureView),
}

impl TransientTextureView {
    pub fn create_texture_view(&self, context: &PassContext) -> TextureView {
        match self {
            TransientTextureView::Read(desc) => desc.create_gpu_texture_view(context),
            TransientTextureView::Write(desc) => desc.create_gpu_texture_view(context),
            TransientTextureView::Owned(texture_view) => texture_view.clone(),
        }
    }
}
