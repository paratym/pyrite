use std::any::TypeId;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

use crate::resource::{BoxedResource, Res, Resource, ResourceBank};
use crate::scheduler::SystemScheduler;
use crate::system::{BoxedSystem, SystemFunction, SystemFunctionHandler};

pub struct AppBuilder {
    pub(crate) resources: HashMap<TypeId, RefCell<BoxedResource>>,
    pub(crate) systems: Vec<BoxedSystem>,
    entry_point: Option<Box<dyn FnOnce(Application)>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            systems: Vec::new(),
            entry_point: None,
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

    pub fn set_entry_point<E>(&mut self, entry_point: E)
    where
        E: FnOnce(Application) + 'static,
    {
        self.entry_point = Some(Box::new(entry_point));
    }

    pub fn run(self) {
        let app = Application {
            resource_bank: ResourceBank::new(self.resources),
            system_scheduler: SystemScheduler::new(self.systems),
        };

        self.entry_point.expect("No entry point was defined")(app);
    }
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
