#[derive(Default)]
pub struct RefreshFlow {
    generation: u64,
    timer: Option<u64>,
}

impl RefreshFlow {
    /// Starts a replacement fetch; old fetches and their timers become stale.
    pub fn start(&mut self) -> u64 {
        self.generation += 1;
        self.timer = None;
        self.generation
    }

    pub fn accepts(&self, generation: u64) -> bool {
        generation == self.generation
    }

    pub fn finished(&mut self, generation: u64) {
        if self.accept_finished(generation) {
            self.timer = Some(generation);
        }
    }

    pub fn accept_finished(&self, generation: u64) -> bool {
        self.accepts(generation) && self.timer.is_none()
    }

    pub fn tick(&mut self, generation: u64) -> bool {
        match self.timer {
            Some(pending) if pending == generation => {
                self.timer = None;
                true
            }
            _ => false,
        }
    }

    pub fn pending_timer(&self) -> bool {
        self.timer.is_some()
    }
}
