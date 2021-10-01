use glam::{IVec2, IVec3, Vec2, Vec3};

const SIZE_X: usize = 128;
const SIZE_Y: usize = 1;
const SIZE_Z: usize = 128;

pub struct Blockmap {
    bitmap: [[[bool; SIZE_X]; SIZE_Z]; SIZE_Y],
}

impl Blockmap {
    pub fn new() -> Self {
        Blockmap {
            bitmap: [[[false; SIZE_X]; SIZE_Z]; SIZE_Y],
        }
    }
    pub fn add(&mut self, pos: IVec3, fields: &[&[u8; 16]]) {
        let y = 0;
        for (z, line) in fields.iter().rev().enumerate() {
            for (x, c) in line.iter().enumerate() {
                let x = (x as i32 + pos.x) as usize;
                let z = (z as i32 + pos.z) as usize;
                self.bitmap[y][z][x] = *c == b'1';
            }
        }
    }

    pub fn get_polygons(
        &self,
    ) -> (
        Vec<Vec3>,
        Vec<(
            (i32, f32, f32),
            (i32, f32, f32),
            (i32, f32, f32),
            (i32, f32, f32),
            usize,
        )>,
    ) {
        let mut points = Vec::new();
        let mut polys = Vec::new();
        for y in 0..SIZE_Y {
            for z in 0..SIZE_Z {
                for x in 0..SIZE_X {
                    if !self.bitmap[y][z][x] {
                        continue;
                    }
                    let offs = points.len() as i32;
                    let origin = Vec3::new(x as f32, y as f32, z as f32) * 20.0;

                    points.extend(
                        [
                            Vec3::new(-10.0, -10.0, 10.0),
                            Vec3::new(-10.0, 10.0, 10.0),
                            Vec3::new(10.0, 10.0, 10.0),
                            Vec3::new(10.0, -10.0, 10.0),
                            Vec3::new(-10.0, -10.0, -10.0),
                            Vec3::new(-10.0, 10.0, -10.0),
                            Vec3::new(10.0, 10.0, -10.0),
                            Vec3::new(10.0, -10.0, -10.0),
                        ]
                        .map(|p| p + origin),
                    );
                    // #[rustfmt::skip]
                    let tw = 256.0;
                    let th = 256.0;
                    if !self.bitmap[y][z + 1][x] {
                        // back
                        polys.push((
                            (offs + 0, 0.0, 0.0),
                            (offs + 1, 0.0, th),
                            (offs + 2, tw, th),
                            (offs + 3, tw, 0.0),
                            0,
                        ));
                    }
                    if !self.bitmap[y][z - 1][x] {
                        // front
                        polys.push((
                            (offs + 7, 0.0, 0.0),
                            (offs + 6, 0.0, th),
                            (offs + 5, tw, th),
                            (offs + 4, tw, 0.0),
                            0,
                        ));
                    }

                    if !self.bitmap[y][z][x - 1] {
                        // left
                        polys.push((
                            (offs + 0, 0.0, 0.0),
                            (offs + 4, tw, 0.0),
                            (offs + 5, tw, th),
                            (offs + 1, 0.0, th),
                            0,
                        ));
                    }
                    if !self.bitmap[y][z][x + 1] {
                        // right
                        polys.push((
                            (offs + 3, 0.0, 0.0),
                            (offs + 2, 0.0, th),
                            (offs + 6, tw, th),
                            (offs + 7, tw, 0.0),
                            0,
                        ));
                    }
                    // top
                    polys.push((
                        (offs + 0, 0.0, 0.0),
                        (offs + 3, 0.0, th),
                        (offs + 7, tw, th),
                        (offs + 4, tw, 0.0),
                        0,
                    ));
                    // bottom
                    polys.push((
                        (offs + 1, 0.0, 0.0),
                        (offs + 5, tw, 0.0),
                        (offs + 6, tw, th),
                        (offs + 2, 0.0, th),
                        1,
                    ));
                }
            }
        }

        (points, polys)
    }
}
