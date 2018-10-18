use std::time::Duration;

pub struct SumWindow {
    history: Vec<Duration>,
    idx: usize,
    sum: Duration,
}

impl SumWindow {
    pub fn new(window_size: usize) -> Self {
        let mut history = Vec::new();
        history.resize(window_size, Duration::new(0, 0));
        SumWindow {
            history: history,
            idx: 0,
            sum: Duration::new(0, 0),
        }
    }

    pub fn tick(&mut self, d: Duration) -> f64 {
        self.sum += d;
        self.sum -= self.history[self.idx];
        self.history[self.idx] = d;
        self.idx = if self.idx + 1 == self.history.len() {
            0
        } else {
            self.idx + 1
        };
        let elapsed = self.sum.as_secs() as f64 + self.sum.subsec_nanos() as f64 * 1e-9;
        self.history.len() as f64 / elapsed
    }
}
