use std::ops::Sub;

// auto [PerspectiveProject, PerspectiveUnproject] = [](int W,int H, float fov)
// {
//     std::tuple center{ W*.5f, H*.5f }, size = center, aspect{ 1.f, W*1.f/H };
//     auto scale = Mul(size, aspect, 1.f / std::tan(fov/2.f * (std::atan(1.f)/45.f)));
//     // Converting 3D x,y,z into 2D X & Y follows this formula (rectilinear projection):
//     //    X = xcenter + x * hscale / z
//     //    Y = ycenter + y * vscale / z
//     return std::pair{ [=](const auto& point) // PerspectiveProject function
//     {
//         return Mul<std::plus<void>>(Mul(scale, point, 1 / std::get<2>(point)), center);
//     },
//     // Doing the same in reverse, getting 3D x,y,z from 2D X & Y,
//     // can be done as follows, but requires that we already know z:
//     //    x = (X - xcenter) * z / hscale
//     //    y = (Y - ycenter) * z / vscale
//     [=](const auto& point, float z) // PerspectiveUnproject function
//     {
//         return std::tuple_cat(Mul<std::divides<void>>(Mul(Mul<std::minus<void>>(point, center), z), scale), std::tuple{z});
//     } };
// }(W,H, 120.f /* degrees */);

// type Vec3f = (f32, f32, f32);
// type Vec2f = (f32, f32);

#[derive(Debug, Clone, Copy)]
pub struct Vec3f(pub f32, pub f32, pub f32);

#[derive(Debug, Clone, Copy)]
pub struct Vec2f(pub f32, pub f32);

impl Sub for Vec2f {
    type Output = Vec2f;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2f(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Sub for Vec3f {
    type Output = Vec3f;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3f(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

fn perspective_project(point: Vec3f, w: f32, h: f32, fov: f32) -> Vec2f {
    let center = (w * 0.5, h * 0.5);
    let size = center;
    let aspect = (1.0, w * 1.0 / h);
    let scale = 1.0 / f32::tan(fov / 2.0 * (f32::atan(1.0) / 45.0));
    let scale = (size.0 * aspect.0 * scale, size.1 * aspect.1 * scale);
    Vec2f(
        center.0 + point.0 * scale.0 / point.2,
        center.1 + point.1 * scale.1 / point.2,
    )
}

fn perspective_unproject(point: Vec2f, z: f32, w: f32, h: f32, fov: f32) -> Vec3f {
    let center = (w * 0.5, h * 0.5);
    let size = center;
    let aspect = (1.0, w * 1.0 / h);
    let scale = 1.0 / f32::tan(fov / 2.0 * (f32::atan(1.0) / 45.0));
    let scale = (size.0 * aspect.0 * scale, size.1 * aspect.1 * scale);

    Vec3f(
        (point.0 - center.0) * z / scale.0,
        (point.1 - center.1) * z / scale.1,
        z,
    )
}

pub fn perspective(
    w: f32,
    h: f32,
    fov: f32,
) -> (impl Fn(Vec3f) -> Vec2f, impl Fn(Vec2f, f32) -> Vec3f) {
    let center = (w * 0.5, h * 0.5);
    let size = center;
    let aspect = (1.0, w * 1.0 / h);
    let scale = 1.0 / f32::tan(fov / 2.0 * (f32::atan(1.0) / 45.0));
    let scale = (size.0 * aspect.0 * scale, size.1 * aspect.1 * scale);

    let (proj, unproj) = (
        move |point: Vec3f| {
            Vec2f(
                center.0 + point.0 * scale.0 / point.2,
                center.1 + point.1 * scale.1 / point.2,
            )
        },
        move |point: Vec2f, z: f32| {
            Vec3f(
                (point.0 - center.0) * z / scale.0,
                (point.1 - center.1) * z / scale.1,
                z,
            )
        },
    );
    (proj, unproj)
}

#[test]
fn test_perspective() {
    let (perspective_project, perspective_unproject) = perspective(800.0, 600.0, 120.0);

    let point3 = Vec3f(10.0, 20.0, 30.0);
    let point2 = perspective_project(point3);
    let point3_un = perspective_unproject(point2, 30.0);
    println!("{:?} {:?} {:?}", point3, point2, point3_un);

    // println!("{:?}", perspective_project((10.0, 10.0, 5.0)));
}

// fn perspective<P, U>(w: f32, h: f32, fov: f32) -> (P, U)
// where
//     P: Fn(Vec3f) -> Vec2f,
//     U: Fn(Vec2f) -> Vec3f,
// {
//     (
//         |point| perspective_project(point, w, h, fov),
//         |point| perspective_unproject(w, h, fov),
//     )
// }
