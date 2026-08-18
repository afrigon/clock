use std::time::{Duration, Instant};

use crate::time_format::format_seconds;

pub struct Stopwatch {
    accumulated: Duration,
    running_since: Option<Instant>,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            accumulated: Duration::ZERO,
            running_since: None,
        }
    }

    pub fn toggle(&mut self) {
        match self.running_since.take() {
            Some(since) => self.accumulated += since.elapsed(),
            None => self.running_since = Some(Instant::now()),
        }
    }

    pub fn reset(&mut self) {
        self.accumulated = Duration::ZERO;
        self.running_since = None;
    }

    pub fn display(&self) -> String {
        format_seconds(self.elapsed().as_secs() as i64)
    }

    pub fn milliseconds_part(&self) -> u32 {
        self.elapsed().subsec_millis()
    }

    fn elapsed(&self) -> Duration {
        let running = self
            .running_since
            .map_or(Duration::ZERO, |since| since.elapsed());
        self.accumulated + running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_paused_at_zero() {
        assert_eq!(Stopwatch::new().display(), "00:00");
    }

    #[test]
    fn accumulates_across_pauses() {
        let mut stopwatch = Stopwatch::new();
        stopwatch.accumulated = Duration::from_secs(65);
        assert_eq!(stopwatch.display(), "01:05");
        stopwatch.accumulated = Duration::from_secs(3600);
        assert_eq!(stopwatch.display(), "01:00:00");
    }

    #[test]
    fn reset_returns_to_zero() {
        let mut stopwatch = Stopwatch::new();
        stopwatch.accumulated = Duration::from_secs(65);
        stopwatch.toggle();
        stopwatch.reset();
        assert_eq!(stopwatch.display(), "00:00");
    }
}
