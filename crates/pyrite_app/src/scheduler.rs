use crate::resource::ResourceBank;
use crate::system::BoxedSystem;

pub struct SystemScheduler {
    systems: Vec<BoxedSystem>,
}

impl SystemScheduler {
    pub(crate) fn new(systems: Vec<BoxedSystem>) -> Self {
        Self { systems }
    }

    // TODO: Make multithreaded in the future
    pub(crate) fn execute(&mut self, resource_bank: &ResourceBank) {
        for system in &mut self.systems {
            system.run(resource_bank);
        }
    }
}
