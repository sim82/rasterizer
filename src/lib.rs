#![feature(step_trait)]

pub mod math;
pub mod rasterize;
pub mod slope;
pub mod texture;

pub mod test_texture {
    pub const TW: usize = 256;
    pub const TH: usize = 256;

    pub fn create() -> [u32; TW * TH] {
        let mut bitmap = [0u32; TW * TH];

        for y in 0..TH as i32 {
            for x in 0..TW as i32 {
                let l = (0x1FF
                    >> [x, y, TW as i32 - 1 - x, TH as i32 - 1 - y, 31]
                        .iter()
                        .min()
                        .unwrap()) as i32;

                // std::cmp::min
                let d = std::cmp::min(
                    50,
                    std::cmp::max(
                        0,
                        255 - 50
                            * f32::powf(
                                f32::hypot(
                                    x as f32 / (TW / 2) as f32 - 1.0f32,
                                    y as f32 / (TH / 2) as f32 - 1.0f32,
                                ) * 4.0,
                                2.0f32,
                            ) as i32,
                    ),
                );
                let r = (!x & !y) & 255;
                let g = (x & !y) & 255;
                let b = (!x & y) & 255;
                let color = std::cmp::min(std::cmp::max(r - d, l), 255) * 65536
                    + std::cmp::min(std::cmp::max(g - d, l), 255) * 256
                    + std::cmp::min(std::cmp::max(b - d, l), 255);
                bitmap[y as usize * TW + x as usize] = color as u32;
            }
        }
        bitmap
    }
}
