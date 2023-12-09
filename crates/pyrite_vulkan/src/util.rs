use std::sync::Arc;

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
