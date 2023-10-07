use std::{
    any::TypeId,
    cell::{
        Ref,
        RefCell,
        RefMut,
    },
    collections::HashMap,
};

use downcast::{
    downcast,
    Any,
};

pub use pyrite_app_macros::Resource;

pub(crate) type BoxedResource = Box<dyn Resource>;

pub type Res<'rb, R> = Ref<'rb, R>;
pub type ResMut<'rb, R> = RefMut<'rb, R>;

pub trait Resource: Any + Send + Sync {}
downcast!(dyn Resource);

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
        Ref::map(
            resource_bank
                .resources
                .get(&TypeId::of::<R>())
                .unwrap()
                .borrow(),
            |r| r.downcast_ref().unwrap(),
        )
    }
    fn from_resource_bank_mut(resource_bank: &ResourceBank) -> ResMut<Self> {
        RefMut::map(
            resource_bank
                .resources
                .get(&TypeId::of::<R>())
                .unwrap()
                .borrow_mut(),
            |r| r.downcast_mut().unwrap(),
        )
    }
}

pub struct ResourceBank {
    resources: HashMap<TypeId, RefCell<BoxedResource>>,
}

impl ResourceBank {
    pub(crate) fn new(resources: HashMap<TypeId, RefCell<BoxedResource>>) -> Self {
        Self { resources }
    }

    pub(crate) fn get_resource<R: Resource>(&self) -> Res<R> {
        Ref::map(
            self.resources.get(&TypeId::of::<R>()).unwrap().borrow(),
            |r| r.downcast_ref().unwrap(),
        )
    }

    pub(crate) fn get_resource_mut<R: Resource>(&self) -> ResMut<R>
    where
        R: Resource,
    {
        RefMut::map(
            self.resources.get(&TypeId::of::<R>()).unwrap().borrow_mut(),
            |r| r.downcast_mut().unwrap(),
        )
    }
}
