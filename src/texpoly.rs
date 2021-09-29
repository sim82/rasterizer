use crate::{
    rasterize::{rasterize_polygon, Slope},
    slope::SlopeData,
};

// #[derive(Debug, Default)]
// pub struct SlopeData {
//     begin: f32,
//     step: f32,
// }
// impl SlopeData {
//     fn new(begin: f32, end: f32, num_steps: f32) -> SlopeData {
//         let inv_step = 1.0 / num_steps as f32;
//         SlopeData {
//             begin: begin as f32,
//             step: (end - begin) as f32 * inv_step,
//         }
//     }
// }
// impl Slope for SlopeData {
//     fn get(&self) -> f32 {
//         self.begin
//     }

//     fn advance(&mut self) {
//         self.begin += self.step;
//     }
// }

// type Point = [i32; 5];
type Point = (f32, f32, f32, f32, f32);

pub fn draw_polygon<F>(points: &[Point], mut fragment: F)
where
    F: FnMut(i32, i32, f32, f32, f32, u32),
{
    // let points = [p0, p1, p2];
    rasterize_polygon(
        &points,
        |p| (p.0, p.1),
        // slope generator
        |from, to, num_steps| {
            let zbegin = 1.0 / from.2;
            let zend = 1.0 / to.2;
            let result = [
                SlopeData::new(from.0, to.0, num_steps),
                SlopeData::new(zbegin, zend, num_steps), // inverted z coordinate
                SlopeData::new(from.3 * zbegin, to.3 * zend, num_steps),
                SlopeData::new(from.4 * zbegin, to.4 * zend, num_steps),
                // SlopeData::new(from.4, to.4, num_steps),
            ];
            result
        },
        //scanline function
        |y, left, right, aux| {
            let xstart = left[0].get();
            let xend = right[0].get();

            let num_steps = xend - xstart;
            let mut props = [
                SlopeData::new(left[1].get(), right[1].get(), num_steps),
                SlopeData::new(left[2].get(), right[2].get(), num_steps),
                SlopeData::new(left[3].get(), right[3].get(), num_steps),
            ];
            for x in (xstart as i32)..(xend as i32) {
                let z = 1.0 / props[0].get();
                fragment(x, y, z, props[1].get() * z, props[2].get() * z, aux);
                for prop in props.iter_mut() {
                    prop.advance();
                }
            }
            for border in left.iter_mut() {
                border.advance();
            }
            for border in right.iter_mut() {
                border.advance();
            }
        },
    )
}
