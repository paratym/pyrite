use std::time::{Duration, Instant};

use pyrite_app::resource::Resource;

#[derive(Resource)]
pub struct Time {
    delta: Duration,
    last: Instant,
}

impl Time {
    pub fn new() -> Self {
        Self {
            delta: Duration::from_secs(0),
            last: Instant::now(),
        }
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last);
        self.last = now;
    }
}
