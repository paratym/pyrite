use std::{any::TypeId, collections::HashMap};

use downcast::{downcast, Any};

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
pub use pyrite_app_macros::Resource;

pub(crate) type BoxedResource = Box<dyn Resource>;

pub type Res<'rb, R> = MappedRwLockReadGuard<'rb, R>;
pub type ResMut<'rb, R> = MappedRwLockWriteGuard<'rb, R>;

pub trait Resource: Any + Send + Sync {}
downcast!(dyn Resource);

// reemove this and just put it in the ystem param implementation because there is no need to
// define it twice and the system param implemntation defines the dependencies.
pub(crate) trait FromResourceBank
where
    Self: Sized,
{
    fn from_resource_bank(resource_bank: &ResourceBank) -> Res<Self>;
    fn from_resource_bank_mut(resource_bank: &ResourceBank) -> ResMut<Self>;
}

impl<R> FromResourceBank for R
where
    R: Resource,
{
    fn from_resource_bank(resource_bank: &ResourceBank) -> Res<Self> {
        RwLockReadGuard::map(
            resource_bank
                .resources
                .get(&TypeId::of::<R>())
                .expect(&format!(
                    "Resource {} is not in the resource bank.",
                    std::any::type_name::<R>()
                ))
                .read(),
            |r| r.downcast_ref().unwrap(),
        )
    }
    fn from_resource_bank_mut(resource_bank: &ResourceBank) -> ResMut<Self> {
        RwLockWriteGuard::map(
            resource_bank
                .resources
                .get(&TypeId::of::<R>())
                .expect(&format!(
                    "Resource {} is not in the resource bank.",
                    std::any::type_name::<R>()
                ))
                .write(),
            |r| r.downcast_mut().unwrap(),
        )
    }
}

pub struct ResourceBank {
    resources: HashMap<TypeId, RwLock<BoxedResource>>,
}

impl ResourceBank {
    pub fn new(resources: HashMap<TypeId, RwLock<BoxedResource>>) -> Self {
        Self { resources }
    }

    pub fn get_resource<R: Resource>(&self) -> Res<R> {
        RwLockReadGuard::map(
            self.resources.get(&TypeId::of::<R>()).unwrap().read(),
            |r| r.downcast_ref().unwrap(),
        )
    }

    pub fn get_resource_mut<R: Resource>(&self) -> ResMut<R>
    where
        R: Resource,
    {
        RwLockWriteGuard::map(
            self.resources.get(&TypeId::of::<R>()).unwrap().write(),
            |r| r.downcast_mut().unwrap(),
        )
    }
}
