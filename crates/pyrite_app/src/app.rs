use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use crate::{
    resource::{BoxedResource, Res, Resource, ResourceBank},
    scheduler::{LinearSystemScheduler, SystemScheduler},
    stage::{Stage, StageBuilder, DEFAULT_STAGE},
    system::SystemFunctionHandler,
};

pub struct AppBuilder {
    pub(crate) resources: HashMap<TypeId, RefCell<BoxedResource>>,
    pub(crate) stages: HashMap<String, StageBuilder>,
    entry_point: Option<Box<dyn FnOnce(Application)>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        let mut new = Self {
            resources: HashMap::new(),
            stages: HashMap::new(),
            entry_point: None,
        };

        new.create_stage(DEFAULT_STAGE);
        return new;
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

    pub fn create_stage(&mut self, name: impl ToString) {
        let name = name.to_string().to_ascii_lowercase();
        let stage_builder = StageBuilder::new(name.clone());

        if self.stages.insert(name.clone(), stage_builder).is_some() {
            panic!("Stage with name '{}' already exists", name);
        }
    }

    pub fn get_stage(&mut self, name: impl ToString) -> &mut StageBuilder {
        let name = name.to_string().to_ascii_lowercase();
        self.stages
            .get_mut(&name)
            .expect(&format!("Stage with name '{}' does not exist", name))
    }

    pub fn add_system<M: 'static>(&mut self, system: impl SystemFunctionHandler<M> + 'static) {
        self.get_stage(DEFAULT_STAGE).add_system(system);
    }

    pub fn add_system_to_stage<M: 'static>(
        &mut self,
        system: impl SystemFunctionHandler<M> + 'static,
        stage: impl ToString,
    ) {
        self.get_stage(stage).add_system(system);
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
            system_scheduler: Box::new(LinearSystemScheduler::new()),
            stages: self
                .stages
                .into_iter()
                .map(|(name, stage)| (name, stage.build()))
                .collect(),
        };

        self.entry_point.expect("No entry point was defined")(app);
    }
}

pub struct Application {
    resource_bank: ResourceBank,
    system_scheduler: Box<dyn SystemScheduler>,
    stages: HashMap<String, Stage>,
}

impl Application {
    pub fn get_resource<R: Resource>(&self) -> Res<R> {
        self.resource_bank.get_resource()
    }

    pub fn get_resource_mut<R: Resource>(&self) -> RefMut<R> {
        self.resource_bank.get_resource_mut()
    }

    pub fn execute_stage(&mut self, stage: impl ToString) {
        let stage_name = stage.to_string().to_ascii_lowercase();
        self.system_scheduler.execute_stage(
            self.stages.get_mut(&stage_name).expect(&format!(
                "Tried to execute a stage that does not exist: {}",
                stage_name
            )),
            &self.resource_bank,
        );
    }
}
