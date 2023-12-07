use std::sync::Arc;

pub type VulkanGenericResourceDep = Arc<dyn VulkanResource>;

pub trait VulkanResource: Send + Sync + 'static {}
pub trait GenericVulkanResourceDep {
    fn into_generic(&self) -> VulkanGenericResourceDep;
}

impl<R> GenericVulkanResourceDep for Arc<R>
where
    R: VulkanResource,
{
    fn into_generic(&self) -> VulkanGenericResourceDep {
        Arc::clone(self) as Arc<dyn VulkanResource>
    }
}
