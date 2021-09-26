use crate::rasterize::Slope;

#[derive(Debug)]
pub struct SlopeData {
    begin: f32,
    step: f32,
}
impl SlopeData {
    pub fn new(begin: f32, end: f32, num_steps: i32) -> SlopeData {
        let inv_step = 1.0 / num_steps as f32;
        SlopeData {
            begin: begin as f32,
            step: (end - begin) as f32 * inv_step,
        }
    }
}
impl Slope for SlopeData {
    fn get(&self) -> f32 {
        self.begin
    }

    fn advance(&mut self) {
        self.begin += self.step;
    }
}
