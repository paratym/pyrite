use crate::resource::{BoxedResource, Res, Resource, ResourceBank};
use crate::scheduler::SystemScheduler;
use crate::system::{BoxedSystem, SystemFunction, SystemFunctionHandler};
use std::any::TypeId;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

pub struct AppBuilder {
    pub(crate) resources: HashMap<TypeId, RefCell<BoxedResource>>,
    pub(crate) systems: Vec<BoxedSystem>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            systems: Vec::new(),
        }
    }

    pub fn add_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.resources
            .insert(TypeId::of::<R>(), RefCell::new(Box::new(resource)));
        self
    }

    pub fn get_resource<R: Resource>(&self) -> Res<R> {
        Ref::map(
            self.resources.get(&TypeId::of::<R>()).unwrap().borrow(),
            |r| r.downcast_ref().unwrap(),
        )
    }

    pub fn get_resource_mut<R: Resource>(&self) -> RefMut<R>
    where
        R: Resource,
    {
        RefMut::map(
            self.resources.get(&TypeId::of::<R>()).unwrap().borrow_mut(),
            |r| r.downcast_mut().unwrap(),
        )
    }

    pub fn add_system<M: 'static>(&mut self, system: impl SystemFunctionHandler<M> + 'static) {
        self.systems.push(SystemFunction::new_boxed(system));
    }

    pub fn run<E>(self)
    where
        E: EntryPoint + 'static,
    {
        E::run(Application {
            resource_bank: ResourceBank::new(self.resources),
            system_scheduler: SystemScheduler::new(self.systems),
        });
    }
}

pub trait EntryPoint {
    fn run(application: Application);
}

pub struct Application {
    resource_bank: ResourceBank,
    system_scheduler: SystemScheduler,
}

impl Application {
    pub fn get_resource<R: Resource>(&self) -> Res<R> {
        self.resource_bank.get_resource()
    }

    pub fn get_resource_mut<R: Resource>(&self) -> RefMut<R> {
        self.resource_bank.get_resource_mut()
    }

    pub fn execute_systems(&mut self) {
        self.system_scheduler.execute(&self.resource_bank);
    }
}
