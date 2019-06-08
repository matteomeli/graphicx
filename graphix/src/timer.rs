#[derive(Default)]
pub struct Timer {}

impl Timer {
    pub fn tick<F>(&self, update: F)
    where
        F: Fn() -> (),
    {
        update();
    }
}
