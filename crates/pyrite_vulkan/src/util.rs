use std::sync::Arc;

use ash::vk;

pub type GenericResourceDep = Arc<dyn VulkanResource>;
pub type WeakGenericResourceDep = std::sync::Weak<dyn VulkanResource>;

pub trait VulkanResource: Send + Sync + 'static {}
pub trait VulkanResourceDep {
    fn into_generic(&self) -> GenericResourceDep;
    fn into_generic_weak(&self) -> WeakGenericResourceDep;
}

impl<R> VulkanResourceDep for Arc<R>
where
    R: VulkanResource,
{
    fn into_generic(&self) -> GenericResourceDep {
        self.clone() as Arc<dyn VulkanResource>
    }

    fn into_generic_weak(&self) -> WeakGenericResourceDep {
        Arc::downgrade(self) as std::sync::Weak<dyn VulkanResource>
    }
}

impl<R> VulkanResourceDep for std::sync::Weak<R>
where
    R: VulkanResource,
{
    fn into_generic(&self) -> GenericResourceDep {
        self.upgrade()
            .expect("Tried to upgrade a weak resource that was already dropped.")
            as Arc<dyn VulkanResource>
    }

    fn into_generic_weak(&self) -> WeakGenericResourceDep {
        self.clone() as std::sync::Weak<dyn VulkanResource>
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Into<vk::Extent2D> for Extent2D {
    fn into(self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.width,
            height: self.height,
        }
    }
}

impl Into<vk::Extent3D> for Extent3D {
    fn into(self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }
}

impl From<vk::Extent2D> for Extent3D {
    // Extracts the width and height and sets the depth to 1.
    fn from(extent: vk::Extent2D) -> Self {
        Self {
            width: extent.width,
            height: extent.height,
            depth: 1,
        }
    }
}

impl From<vk::Extent3D> for Extent2D {
    // Extracts the width and height.
    fn from(extent: vk::Extent3D) -> Self {
        Self {
            width: extent.width,
            height: extent.height,
        }
    }
}
