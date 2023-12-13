use std::{any::TypeId, collections::HashMap};

use crate::system::{BoxedSystem, SystemFunction, SystemFunctionHandler, SystemParam};

pub struct ScheduleSystemConfig {
    name: String,
    system_dependencies: Vec<String>,
    boxed_system: BoxedSystem,
}

pub struct ScheduleBuilder {
    systems: Vec<ScheduleSystemConfig>,
}

impl ScheduleBuilder {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_task<T: ScheduleTask<M> + 'static, M>(&mut self, schedule_task: T) {
        let system = schedule_task.into_boxed_system();
        let system_dependencies = T::collect_dependencies();

        println!("Added system: {}", system.name());
        println!("with dependencies: {:?}", system_dependencies);

        self.systems.push(ScheduleSystemConfig {
            name: system.name().to_string(),
            system_dependencies,
            boxed_system: system,
        });
    }

    pub fn build(self) -> Schedule {
        let systems = self
            .systems
            .into_iter()
            .map(|system_config| system_config.boxed_system)
            .collect::<Vec<_>>();

        Schedule {
            systems,
            system_dependencies: HashMap::new(),
            system_resource_dependencies: HashMap::new(),
        }
    }
}

pub struct Schedule {
    systems: Vec<BoxedSystem>,
    system_dependencies: HashMap<u32, Vec<u32>>,
    system_resource_dependencies: HashMap<u32, Vec<TypeId>>,
}

impl Schedule {
    pub fn systems_mut(&mut self) -> &mut Vec<BoxedSystem> {
        &mut self.systems
    }

    pub fn system_dependencies(&self) -> &HashMap<u32, Vec<u32>> {
        &self.system_dependencies
    }

    pub fn system_resource_dependencies(&self) -> &HashMap<u32, Vec<TypeId>> {
        &self.system_resource_dependencies
    }
}

pub trait ScheduleTask<Marker> {
    fn into_boxed_system(self) -> BoxedSystem;
    fn collect_dependencies() -> Vec<String>;
}

impl<F, M: 'static> ScheduleTask<M> for F
where
    F: SystemFunctionHandler<M> + 'static,
{
    fn into_boxed_system(self) -> BoxedSystem {
        SystemFunction::new_boxed(self)
    }
    fn collect_dependencies() -> Vec<String> {
        vec![]
    }
}

impl<F, M: 'static, S, SM> ScheduleTask<(M, SM)> for (F, S)
where
    F: SystemFunctionHandler<M> + 'static,
    S: ScheduleTaskDependency<SM>,
{
    fn into_boxed_system(self) -> BoxedSystem {
        SystemFunction::new_boxed(self.0)
    }
    fn collect_dependencies() -> Vec<String> {
        S::collect_dependencies()
    }
}

pub trait ScheduleTaskDependency<M> {
    fn collect_dependencies() -> Vec<String>;
}

impl<F: 'static, M> ScheduleTaskDependency<M> for F
where
    F: SystemFunctionHandler<M>,
{
    fn collect_dependencies() -> Vec<String> {
        vec![std::any::type_name::<F>().to_string()]
    }
}

impl<S1, S1M, S2, S2M> ScheduleTaskDependency<fn(S1M, S2M) -> ()> for (S1, S2)
where
    S1: ScheduleTaskDependency<S1M>,
    S2: ScheduleTaskDependency<S2M>,
{
    fn collect_dependencies() -> Vec<String> {
        let mut dependencies = S1::collect_dependencies();
        dependencies.extend(S2::collect_dependencies());
        dependencies
    }
}
