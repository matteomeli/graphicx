use crate::timer::Timer;

#[derive(Default)]
pub struct Engine {
    timer: Timer,
}

impl Engine {
    pub fn tick(&self) {
        self.timer.tick(|| {
            self.update(&self.timer);
        });

        self.draw();
    }

    fn update(&self, _timer: &Timer) {}

    fn draw(&self) {}
}
