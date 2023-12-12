use std::collections::HashMap;

use crate::system::{BoxedSystem, SystemFunction, SystemFunctionHandler, SystemParam};

pub struct ScheduleBuilder {
    systems: HashMap<usize, BoxedSystem>,
}

impl ScheduleBuilder {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
        }
    }

    pub fn add_task<M>(&mut self, schedule_task: impl ScheduleTask<M> + 'static) {
        let system = schedule_task.into_boxed_system();
        let index = self.systems.len();
        self.systems.insert(index, system);
    }

    pub fn build(self) -> Schedule {
        Schedule::new()
    }
}

pub struct Schedule {}

impl Schedule {
    pub fn new() -> Self {
        Self {}
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
        vec![]
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
