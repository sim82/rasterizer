#![feature(step_trait)]

use glam::Vec3;

pub mod math;
pub mod rasterize;
pub mod slope;
pub mod texpoly;
pub mod texpoly_vec;

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
#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let normal = (b - a).cross(c - a).normalize();
        let distance = normal.dot(a);
        Plane { normal, distance }
    }

    pub fn distance_to(&self, p: Vec3) -> f32 {
        self.normal.dot(p) - self.distance
    }
}

#[test]
fn test_plane() {
    let p = Plane::new(
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(2.0, 1.0, 1.0),
        Vec3::new(1.0, 2.0, 1.0),
    );
    println!("{:?}", p);

    let p = Plane::new(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    );
    println!("{:?}", p);

    let p = Plane::new(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 1.0),
    );
    println!("{:?}", p);
}

pub fn clip_polygon(plane: Plane, points: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::new();
    let mut keepfirst = true;

    for i in 0..points.len() {
        let current = points[i];
        let next = if i < points.len() - 1 {
            points[i + 1]
        } else {
            points[0]
        };

        let outside = plane.distance_to(current);
        let outside_next = plane.distance_to(next);
        let mut keep = outside >= 0.0;
        if i == 0 {
            keepfirst = keep;
            keep = true;
        }
        if (outside < 0.0 && outside_next > 0.0) || (outside > 0.0 && outside_next < 0.0) {
            let factor = outside / (outside - outside_next);
            let b = current + (next - current) * factor;

            if keep {
                out.push(current);
            }
            out.push(b);
        } else {
            if keep {
                out.push(current);
            }
        }
    }
    if !keepfirst {
        out.remove(0);
    }
    out
}

#[test]
pub fn test_clip() {
    let points = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(2.0, 2.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
    ];

    let plane = Plane::new(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 1.0),
    );

    let clipped = clip_polygon(plane, &points);
    println!("{:?}", clipped);

    let plane = Plane::new(
        Vec3::new(0.0, 1.0, 1.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
    );

    let clipped = clip_polygon(plane, &points);
    println!("{:?}", clipped);
}
