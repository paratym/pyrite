use crate::{resource::ResourceBank, stage::Stage};

pub trait SystemScheduler {
    fn execute_stage(&mut self, stage: &mut Stage, resource_bank: &ResourceBank);
}

/// A linear system scheduler executes systems in the order they are added to the stage.
/// This schedular runs the systems in a single thread.
pub struct LinearSystemScheduler {}

impl LinearSystemScheduler {
    pub fn new() -> Self {
        Self {}
    }
}

impl SystemScheduler for LinearSystemScheduler {
    fn execute_stage(&mut self, stage: &mut Stage, resource_bank: &ResourceBank) {
        // println!("[pyrite_app]: Executing stage - {}", stage.name());
        for system in stage.systems_mut() {
            // println!("[pyrite_app]: Executing system - {}", system.name());
            system.run(resource_bank);
        }
    }
}
