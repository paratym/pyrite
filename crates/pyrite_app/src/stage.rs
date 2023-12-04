use crate::system::{BoxedSystem, SystemFunction, SystemFunctionHandler};

pub const DEFAULT_STAGE: &'static str = "default";

pub struct Stage {
    name: String,
    systems: Vec<BoxedSystem>,
}

impl Stage {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn systems(&self) -> &[BoxedSystem] {
        &self.systems
    }

    pub(crate) fn systems_mut(&mut self) -> &mut [BoxedSystem] {
        &mut self.systems
    }
}

pub struct StageBuilder {
    name: String,
    systems: Vec<BoxedSystem>,
}

impl StageBuilder {
    pub(crate) fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            systems: Vec::new(),
        }
    }

    pub fn add_system<M: 'static>(&mut self, system: impl SystemFunctionHandler<M> + 'static) {
        self.systems.push(SystemFunction::new_boxed(system));
    }

    pub(crate) fn build(self) -> Stage {
        Stage {
            name: self.name,
            systems: self.systems,
        }
    }
}
