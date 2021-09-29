use crate::rasterize::Slope;

#[derive(Debug, Default)]
pub struct SlopeData {
    pub begin: f32,
    step: f32,
}
impl SlopeData {
    #[inline(always)]
    pub fn new(begin: f32, end: f32, num_steps: f32) -> SlopeData {
        let inv_step = 1.0 / num_steps;
        SlopeData {
            begin: begin,
            step: (end - begin) as f32 * inv_step,
        }
    }
}
impl Slope<f32> for SlopeData {
    #[inline(always)]
    fn get(&self) -> f32 {
        self.begin
    }
    #[inline(always)]
    fn advance(&mut self) {
        self.begin += self.step;
    }
}

#[derive(Debug, Default)]
pub struct Slope4x {
    pub begin: glam::Vec4,
    pub step: glam::Vec4,
}

impl Slope4x {
    pub fn new(begin: glam::Vec4, end: glam::Vec4, num_steps: glam::Vec4) -> Self {
        let inv_step = glam::Vec4::splat(1.0) / num_steps;
        Slope4x {
            begin,
            step: (end - begin) * inv_step,
        }
    }
}
impl Slope<glam::Vec4> for Slope4x {
    #[inline(always)]
    fn get(&self) -> glam::Vec4 {
        self.begin
    }
    #[inline(always)]
    fn advance(&mut self) {
        self.begin += self.step;
    }
}
