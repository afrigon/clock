use std::time::{Duration, Instant};

pub struct Timer {
    target: Duration,
    accumulated: Duration,
    running_since: Option<Instant>,
}

impl Timer {
    pub fn new(target: Duration) -> Self {
        Self {
            target,
            accumulated: Duration::ZERO,
            running_since: Some(Instant::now()),
        }
    }

    pub fn restart(&mut self) {
        self.accumulated = Duration::ZERO;
        self.running_since = Some(Instant::now());
    }

    pub fn toggle_paused(&mut self) {
        match self.running_since.take() {
            Some(since) => self.accumulated += since.elapsed(),
            None => self.running_since = Some(Instant::now()),
        }
    }

    pub fn is_paused(&self) -> bool {
        self.running_since.is_none()
    }

    pub fn remaining_milliseconds(&self) -> i64 {
        let running = self
            .running_since
            .map_or(Duration::ZERO, |since| since.elapsed());
        self.target.as_millis() as i64 - (self.accumulated + running).as_millis() as i64
    }

    pub fn display(&self) -> String {
        format(self.remaining_milliseconds())
    }
}

pub fn format(milliseconds: i64) -> String {
    crate::time_format::format_seconds((milliseconds + 999).div_euclid(1000))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_remaining_rounds_up() {
        assert_eq!(format(2500), "00:03");
        assert_eq!(format(1), "00:01");
    }

    #[test]
    fn zero_crossing() {
        assert_eq!(format(0), "00:00");
        assert_eq!(format(-999), "00:00");
        assert_eq!(format(-1000), "-00:01");
        assert_eq!(format(-1999), "-00:01");
        assert_eq!(format(-2000), "-00:02");
    }

    #[test]
    fn hours_appear_at_one_hour() {
        assert_eq!(format(3_600_000), "01:00:00");
        assert_eq!(format(5_410_000), "01:30:10");
    }

    #[test]
    fn hours_drop_below_one_hour() {
        assert_eq!(format(3_599_000), "59:59");
    }

    #[test]
    fn overtime_grows_into_hours() {
        assert_eq!(format(-3_599_999), "-59:59");
        assert_eq!(format(-3_600_000), "-01:00:00");
    }

    #[test]
    fn pause_resume_and_restart() {
        let mut timer = Timer::new(Duration::from_secs(10));
        assert!(!timer.is_paused());
        timer.toggle_paused();
        assert!(timer.is_paused());
        let frozen = timer.remaining_milliseconds();
        assert_eq!(timer.remaining_milliseconds(), frozen);
        timer.toggle_paused();
        assert!(!timer.is_paused());
        timer.toggle_paused();
        timer.restart();
        assert!(!timer.is_paused());
    }
}
