use std::time;

use pyrite_app::resource::Resource;

#[derive(Resource)]
pub struct Time {
    /// The time of the last frame.
    last_frame: time::Instant,

    /// Elapsed time since the start of the application in seconds.
    elapsed: time::Duration,

    /// Time since the last frame in seconds.
    delta: time::Duration,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_frame: time::Instant::now(),
            elapsed: time::Duration::from_secs(0),
            delta: time::Duration::from_secs(0),
        }
    }

    pub(crate) fn update(&mut self) {
        let now = time::Instant::now();
        self.delta = now.duration_since(self.last_frame);
        self.elapsed += self.delta;
    }

    pub fn elapsed(&self) -> time::Duration {
        self.elapsed
    }

    pub fn delta(&self) -> time::Duration {
        self.delta
    }
}
