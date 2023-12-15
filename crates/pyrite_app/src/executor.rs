use crate::{resource::ResourceBank, schedule::Schedule};

pub struct ScheduleExecutor {
    threads: rayon::ThreadPool,
}

impl ScheduleExecutor {
    pub fn new() -> Self {
        Self {
            threads: rayon::ThreadPoolBuilder::new().build().unwrap(),
        }
    }

    pub fn execute(&mut self, schedule: &mut Schedule, resource_bank: &ResourceBank) {
        for system in schedule.systems_mut() {
            self.threads.install(|| {
                // println!("[pyrite_app]: Executing system - {}", system.name());
                system.run(resource_bank);
            });
        }
    }
}
