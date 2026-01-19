mod buffer;
mod cache;
mod texture;

pub use buffer::*;
pub use cache::*;
pub use texture::*;

use std::{fmt::Debug, sync::Arc};

#[derive(Clone)]
pub enum VirtualResource {
    Setuped(AnyTransientResourceDescriptor),
    Imported(ArcAnyTransientResource),
}

impl VirtualResource {
    pub fn get_desc<ResourceType: TransientResource>(&self) -> ResourceType::Descriptor {
        let desc = match self {
            VirtualResource::Imported(resource) => resource.get_desc(),
            VirtualResource::Setuped(desc) => desc.clone(),
        };

        <ResourceType::Descriptor as TransientResourceDescriptor>::borrow_resource_descriptor(&desc)
            .clone()
    }
}

#[derive(Clone)]
pub enum ArcAnyTransientResource {
    Buffer(Arc<TransientBuffer>),
    Texture(Arc<TransientTexture>),
}

impl ArcAnyTransientResource {
    pub fn get_desc(&self) -> AnyTransientResourceDescriptor {
        match self {
            ArcAnyTransientResource::Buffer(res) => {
                AnyTransientResourceDescriptor::Buffer(res.desc.clone())
            }
            ArcAnyTransientResource::Texture(res) => {
                AnyTransientResourceDescriptor::Texture(res.desc.clone())
            }
        }
    }
}

pub trait IntoArcAnyTransientResource: TransientResource {
    fn into_arc_transient_resource(self: Arc<Self>) -> ArcAnyTransientResource;
}

pub enum AnyTransientResource {
    OwnedBuffer(TransientBuffer),
    ImportedBuffer(Arc<TransientBuffer>),
    OwnedTexture(TransientTexture),
    ImportedTexture(Arc<TransientTexture>),
}

impl From<TransientBuffer> for AnyTransientResource {
    fn from(value: TransientBuffer) -> Self {
        AnyTransientResource::OwnedBuffer(value)
    }
}

impl From<Arc<TransientBuffer>> for AnyTransientResource {
    fn from(value: Arc<TransientBuffer>) -> Self {
        AnyTransientResource::ImportedBuffer(value)
    }
}

impl From<TransientTexture> for AnyTransientResource {
    fn from(value: TransientTexture) -> Self {
        AnyTransientResource::OwnedTexture(value)
    }
}

impl From<Arc<TransientTexture>> for AnyTransientResource {
    fn from(value: Arc<TransientTexture>) -> Self {
        AnyTransientResource::ImportedTexture(value)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum AnyTransientResourceDescriptor {
    Buffer(TransientBufferDescriptor),
    Texture(TransientTextureDescriptor),
}

pub trait TransientResource: 'static {
    type Descriptor: TransientResourceDescriptor;

    fn borrow_resource(res: &AnyTransientResource) -> &Self;

    fn get_desc(&self) -> &Self::Descriptor;
}

pub trait TransientResourceDescriptor:
    'static + Clone + Debug + Into<AnyTransientResourceDescriptor>
{
    type Resource: TransientResource;

    fn borrow_resource_descriptor(res: &AnyTransientResourceDescriptor) -> &Self;
}

pub trait TypeEquals {
    type Other;
    fn same(value: Self) -> Self::Other;
}

impl<T: Sized> TypeEquals for T {
    type Other = Self;
    fn same(value: Self) -> Self::Other {
        value
    }
}

#[cfg(test)]
mod tests {
    use wgpu::BufferUsages;
    use wgpu::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

    use crate::{
        TransientResourceCreator,
        transient_resource::{
            AnyTransientResource, AnyTransientResourceDescriptor, TransientBufferDescriptor,
            TransientTextureDescriptor,
        },
    };

    #[test]
    fn transient_resource_creator_create_buffer() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        }))
        .expect("Failed to get adapter");

        let (device, _) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Test Device"),
            ..Default::default()
        }))
        .expect("Failed to get device");

        let buffer_desc = TransientBufferDescriptor {
            label: Some("test buffer".to_string()),
            size: 1024,
            usage: BufferUsages::VERTEX,
            mapped_at_creation: false,
        };
        let any_desc = AnyTransientResourceDescriptor::Buffer(buffer_desc.clone());

        let resource = device.create_resource(&any_desc);

        match resource {
            AnyTransientResource::OwnedBuffer(buffer) => {
                assert_eq!(buffer.desc.label, buffer_desc.label);
                assert_eq!(buffer.desc.size, buffer_desc.size);
                assert_eq!(buffer.desc.usage, buffer_desc.usage);
            }
            _ => panic!("Expected OwnedBuffer variant"),
        }
    }

    #[test]
    fn transient_resource_creator_create_texture() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        }))
        .expect("Failed to get adapter");

        let (device, _) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Test Device"),
            ..Default::default()
        }))
        .expect("Failed to get device");

        let texture_desc = TransientTextureDescriptor {
            label: Some("test texture".to_string()),
            size: Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            mip_level_count: 4,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING,
            view_formats: vec![],
        };
        let any_desc = AnyTransientResourceDescriptor::Texture(texture_desc.clone());

        let resource = device.create_resource(&any_desc);

        match resource {
            AnyTransientResource::OwnedTexture(texture) => {
                assert_eq!(texture.desc.label, texture_desc.label);
                assert_eq!(texture.desc.size.width, texture_desc.size.width);
                assert_eq!(texture.desc.mip_level_count, texture_desc.mip_level_count);
                assert_eq!(texture.desc.format, texture_desc.format);
            }
            _ => panic!("Expected OwnedTexture variant"),
        }
    }

    #[test]
    fn transient_resource_creator_create_multiple_resources() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        }))
        .expect("Failed to get adapter");

        let (device, _) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Test Device"),
            ..Default::default()
        }))
        .expect("Failed to get device");

        let buffer_desc = TransientBufferDescriptor {
            label: None,
            size: 2048,
            usage: BufferUsages::INDEX,
            mapped_at_creation: false,
        };
        let texture_desc = TransientTextureDescriptor {
            label: None,
            size: Extent3d {
                width: 512,
                height: 512,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![],
        };

        let buffer_resource =
            device.create_resource(&AnyTransientResourceDescriptor::Buffer(buffer_desc));
        let texture_resource =
            device.create_resource(&AnyTransientResourceDescriptor::Texture(texture_desc));

        match buffer_resource {
            AnyTransientResource::OwnedBuffer(_) => {}
            _ => panic!("Expected OwnedBuffer"),
        }

        match texture_resource {
            AnyTransientResource::OwnedTexture(_) => {}
            _ => panic!("Expected OwnedTexture"),
        }
    }
}
