use std::sync::Arc;

pub type GenericResourceDep = Arc<dyn VulkanResource>;

pub trait VulkanResource: Send + Sync + 'static {}
pub trait VulkanResourceDep {
    fn into_generic(&self) -> GenericResourceDep;
}

impl<R> VulkanResourceDep for Arc<R>
where
    R: VulkanResource,
{
    fn into_generic(&self) -> GenericResourceDep {
        Arc::clone(self) as Arc<dyn VulkanResource>
    }
}
