use crate::resource::ResourceBank;
use crate::system::BoxedSystem;
use std::collections::HashMap;

pub struct SystemScheduler {
    stages: HashMap<String, Vec<BoxedSystem>>,
}

impl SystemScheduler {
    pub(crate) fn new(stages: HashMap<String, Vec<BoxedSystem>>) -> Self {
        Self { stages }
    }

    // TODO: Make multithreaded in the future
    pub(crate) fn execute_stage(&mut self, stage: impl ToString, resource_bank: &ResourceBank) {
        let stage_name = stage.to_string().to_ascii_lowercase();
        for system in self.stages.get_mut(&stage_name).unwrap() {
            system.run(resource_bank);
        }
    }
}
