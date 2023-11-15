use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{resource::ResourceBank, system::BoxedSystem};
use std::collections::HashMap;

pub struct SystemScheduler {
    executors: rayon::ThreadPool,
    stages: HashMap<String, Vec<BoxedSystem>>,
}

impl SystemScheduler {
    pub(crate) fn new(stages: HashMap<String, Vec<BoxedSystem>>) -> Self {
        Self {
            executors: rayon::ThreadPoolBuilder::new()
                .num_threads(3)
                .build()
                .unwrap(),
            stages,
        }
    }

    pub(crate) fn execute_stage(&mut self, stage: impl ToString, resource_bank: &ResourceBank) {
        let stage_name = stage.to_string().to_ascii_lowercase();
        let systems = self.stages.get_mut(&stage_name).unwrap();
        self.executors.install(|| {
            systems.par_iter_mut().for_each(|system| {
                // println!("Running system: {}", system.name());
                system.run(resource_bank);
            });
        });
    }
}
