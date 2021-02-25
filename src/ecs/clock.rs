use std::time::{Duration, Instant};

pub struct Clock {
    prev: Option<Instant>,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            prev: None,
        }
    }
    pub fn tick(&mut self) -> Duration {
        if let Some(prev) = self.prev {
            let now = Instant::now();
            let interval = now - prev;
            self.prev = Some(now);
            interval
        } else {
            self.prev = Some(Instant::now());
            Duration::from_secs(0)
        }
    }
}