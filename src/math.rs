pub mod prelude {
    pub use glam::{IVec2, UVec2, Vec2, Vec3};
}

use prelude::*;

pub fn perspective(
    w: f32,
    h: f32,
    fov: f32,
) -> (impl Fn(Vec3) -> Vec2, impl Fn(Vec2, f32) -> Vec3) {
    let center = Vec2::new(w * 0.5, h * 0.5);
    let size = center;
    let aspect = Vec2::new(1.0, w * 1.0 / h);
    let scale = 1.0 / f32::tan(fov / 2.0 * (std::f32::consts::PI / 180.0));
    let scale = Vec2::new(size.x * aspect.x * scale, size.y * aspect.y * scale);

    let (proj, unproj) = (
        move |point: Vec3| {
            Vec2::new(
                center.x + point.x * scale.x / point.z,
                center.y + point.y * scale.y / point.z,
            )
        },
        move |point: Vec2, z: f32| {
            // Doing the same in reverse, getting 3D x,y,z from 2D X & Y,
            // can be done as follows, but requires that we already know z:
            //    x = (X - xcenter) * z / hscale
            //    y = (Y - ycenter) * z / vscale
            Vec3::new(
                (point.x - center.x) * z / scale.x,
                (point.y - center.y) * z / scale.y,
                z,
            )
        },
    );
    (proj, unproj)
}

#[test]
fn test_perspective() {
    let (perspective_project, perspective_unproject) = perspective(800.0, 600.0, 120.0);

    let point3 = Vec3::new(10.0, 20.0, 30.0);
    let point2 = perspective_project(point3);
    let point3_un = perspective_unproject(point2, 30.0);
    println!("{:?} {:?} {:?}", point3, point2, point3_un);

    // println!("{:?}", perspective_project((10.0, 10.0, 5.0)));
}
