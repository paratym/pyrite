use crate::system::{BoxedSystem, SystemFunction, SystemFunctionHandler};

pub const DEFAULT_STAGE: &'static str = "default";

pub struct StageBuilder {
    systems: Vec<BoxedSystem>,
}

impl StageBuilder {
    pub(crate) fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system<M: 'static>(&mut self, system: impl SystemFunctionHandler<M> + 'static) {
        self.systems.push(SystemFunction::new_boxed(system));
    }

    pub(crate) fn build(self) -> Vec<BoxedSystem> {
        self.systems
    }
}
