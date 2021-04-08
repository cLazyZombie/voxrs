use std::time::{Duration, Instant};

pub struct Fps {
    tick_times: Vec<Duration>,
    last_time: Option<Instant>,
    idx: usize,
}

const FPS_SAMPLES: usize = 200;

impl Fps {
    pub fn new() -> Fps {
        Self {
            tick_times: vec![Duration::from_millis(0); FPS_SAMPLES],
            last_time: None,
            idx: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_internal(Instant::now());
    }

    fn tick_internal(&mut self, now: Instant) {
        if let Some(last_time) = self.last_time {
            let duration = now - last_time;
            self.tick_times[self.idx] = duration;
        }

        self.last_time = Some(now);

        self.idx += 1;
        if self.idx >= FPS_SAMPLES {
            self.idx = 0;
        }
    }

    pub fn get_fps(&self) -> f32 {
        let sum: Duration = self.tick_times.iter().sum();
        let avg: Duration = sum / FPS_SAMPLES as u32;
        let avg_sec = avg.as_secs_f32();
        if avg_sec < 0.0001 {
            return 0.0;
        }

        1.0 / avg_sec
    }

    #[cfg(test)]
    pub fn force_tick(&mut self, now: Instant) {
        self.tick_internal(now);
    }
}

impl Default for Fps {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps() {
        let mut fps = Fps::new();

        let mut tick = Instant::now();
        for _ in 0..200 {
            fps.force_tick(tick);
            tick += Duration::from_millis(33);
        }

        let fps = fps.get_fps();
        assert_eq!(fps.floor() as i32, 30);
    }
}
