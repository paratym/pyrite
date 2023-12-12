use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use crate::{
    executor::ScheduleExecutor,
    resource::{BoxedResource, Res, Resource, ResourceBank},
    schedule::Schedule,
    system::SystemFunctionHandler,
};

pub struct AppBuilder {
    resources: HashMap<TypeId, RefCell<BoxedResource>>,
    schedule: Option<Schedule>,
    entry_point: Option<Box<dyn FnOnce(Application)>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            schedule: None,
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

    pub fn set_schedule(&mut self, schedule: impl Into<Schedule>) {
        self.schedule = Some(schedule.into());
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
            schedule_executor: ScheduleExecutor::new(),
            schedule: self.schedule.expect("No schedule was defined"),
        };

        self.entry_point.expect("No entry point was defined")(app);
    }
}

pub struct Application {
    resource_bank: ResourceBank,
    schedule_executor: ScheduleExecutor,
    schedule: Schedule,
}

impl Application {
    pub fn get_resource<R: Resource>(&self) -> Res<R> {
        self.resource_bank.get_resource()
    }

    pub fn get_resource_mut<R: Resource>(&self) -> RefMut<R> {
        self.resource_bank.get_resource_mut()
    }

    pub fn execute_schedule(&mut self) {}
}
