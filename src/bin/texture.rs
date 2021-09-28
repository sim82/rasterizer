#![feature(step_trait)]
use std::{
    collections::VecDeque,
    fmt::Debug,
    time::{Duration, Instant},
};

use num_traits::pow;
use rasterize::{
    math::{self, prelude::*},
    rasterize::{rasterize_triangle, Slope},
    test_texture, texture,
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseWheelDirection, pixels::PixelFormatEnum};

fn main() {
    const WINDOW_SCALE: u32 = 4;
    const W: u32 = 424;
    const H: u32 = 240;

    let blank = 0x0u32;
    let mut pixels = [blank; (W * H) as usize];

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", W * WINDOW_SCALE, H * WINDOW_SCALE)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .software()
        // .present_vsync()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture(
            PixelFormatEnum::ABGR8888,
            sdl2::render::TextureAccess::Streaming,
            W,
            H,
        )
        .unwrap();

    let tw = test_texture::TW as f32;
    let th = test_texture::TH as f32;

    let mut xrect = [-10.0, -10.0, 10.0, 10.0];
    let mut yrect = [-10.0, 10.0, 10.0, -10.0];
    //  let mut zrect = [20.0, 20.0, 5.0, 5.0];

    let mut zrect = [20.0, 20.0, 5.0, 5.0];

    // let mut xrect = [100.0, 100.0, 500.0, 500.0];
    // let mut yrect = [100.0, 500.0, 500.0, 100.0];
    // let mut zrect = [20.0, 20.0, 10.0, 10.0];

    // https://excalidraw.com/#json=4721006165884928,0MUG2eCYGmj706dqxZThow
    let mut points = [
        Vec3::new(-10.0, -10.0, 20.0),
        Vec3::new(-10.0, 10.0, 20.0),
        Vec3::new(10.0, 10.0, 20.0),
        Vec3::new(10.0, -10.0, 20.0),
        Vec3::new(-10.0, -10.0, 5.0),
        Vec3::new(-10.0, 10.0, 5.0),
        Vec3::new(10.0, 10.0, 5.0),
        Vec3::new(10.0, -10.0, 5.0),
    ];
    let mut triangles = vec![
        // back
        ((0, 0.0, 0.0), (1, 0.0, th), (2, tw, th)),
        ((2, tw, th), (3, tw, 0.0), (0, 0.0, 0.0)),
        // left
        ((0, 0.0, 0.0), (1, 0.0, th), (5, tw, th)),
        ((5, tw, th), (4, tw, 0.0), (0, 0.0, 0.0)),
        // right
        ((3, 0.0, 0.0), (2, 0.0, th), (6, tw, th)),
        ((6, tw, th), (7, tw, 0.0), (3, 0.0, 0.0)),
        // top
        ((0, 0.0, 0.0), (3, 0.0, th), (7, tw, th)),
        ((7, tw, th), (4, tw, 0.0), (0, 0.0, 0.0)),
        // bottom
        ((1, 0.0, 0.0), (2, 0.0, th), (6, tw, th)),
        ((6, tw, th), (5, tw, 0.0), (1, 0.0, 0.0)),
    ];
    // let mut new_triangle = VecDeque::new();
    let mut click_index = 0;
    let bitmap = test_texture::create();
    let mut test_texture = texture_creator
        .create_texture(
            PixelFormatEnum::ABGR8888,
            sdl2::render::TextureAccess::Streaming,
            test_texture::TW as u32,
            test_texture::TH as u32,
        )
        .unwrap();

    test_texture
        .update(
            None,
            unsafe { std::mem::transmute::<&[u32], &[u8]>(&bitmap) },
            4 * test_texture::TW,
        )
        .unwrap();

    let l = Vec3::new(0.0, 0.0, -10.0);

    let camera_rot = glam::Mat3::from_rotation_y(1.0);

    let (perspective_project, perspective_unproject) = math::perspective(W as f32, H as f32, 90.0);
    let mut r = 0.0;

    const bayer4x4_f: [[f32; 4]; 4] = [
        // 4x4 ordered-dithering matrix
        [0.0 / 16.0, 8.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
        [12.0 / 16.0, 4.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
        [3.0 / 16.0, 11.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
        [15.0 / 16.0, 7.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
    ];
    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::MouseWheel { y, .. } => {
                    let y = y as f32;
                    for p in points[0..4].iter_mut() {
                        p.z += y;
                    }
                }
                Event::MouseButtonDown { x, y, .. } => {
                    // xrect[click_index % 4] = x as f32;
                    // yrect[click_index % 4] = y as f32;
                    let point = (IVec2::new(x, y) / WINDOW_SCALE as i32).as_vec2();

                    let z = if (click_index % 4) < 2 { 20.0 } else { 5.0 };
                    let z = z - l.z;
                    let (wx, wy, wz) = (perspective_unproject(point, z) + l).into();
                    xrect[click_index % 4] = wx;
                    yrect[click_index % 4] = wy;

                    click_index += 1;
                }
                _ => {}
            }
        }

        let camera_rot = glam::Mat3::from_rotation_y(r);
        // r += std::f32::consts::PI / 180.0;

        let mut color = 0x3b0103a5u32;
        let duplicate = 0xffaa55u32;
        pixels.fill(blank);

        let start = Instant::now();
        let m = Vec3::ZERO;
        // for yt in -5..5 {
        //     for xt in -5..5 {
        //         let m = Vec3::new(xt as f32 * 5.0, yt as f32 * 5.0, 0.0);

        for (p0, p1, p2) in triangles.iter().cloned() {
            color = (color << 1) | (color >> (32 - 1));

            let transform = |(index, u, v)| {
                let p3 = camera_rot * (points[index] - l);
                let (vx, vy) = perspective_project(p3).into();
                (vx, vy, p3.z, u, v)
            };

            // let transform = |p| p;

            texture::draw_polygon(
                transform(p0),
                transform(p1),
                transform(p2),
                |x, y, z, u, v, aux| {
                    let x = x as usize;
                    let y = y as usize;
                    let pixel_index = y * W as usize + x;
                    let mut bad_pixel = 0;
                    let pixel = if pixel_index < pixels.len() {
                        unsafe { pixels.get_unchecked_mut(pixel_index) }
                    } else {
                        &mut bad_pixel
                    };
                    if *pixel != blank {
                        *pixel = duplicate;
                    } else {
                        let ui = (u + bayer4x4_f[y % 4][x % 4]) as usize;
                        let vi = (v + bayer4x4_f[y % 4][x % 4]) as usize;
                        let color = bitmap
                            [(vi % test_texture::TH) * test_texture::TW + (ui % test_texture::TW)];
                        *pixel = color & 0xffffff;
                        // if aux == 0 {
                        //     *pixel = 0xff0000;
                        // } else {
                        //     *pixel = 0xff00;
                        // }
                    }
                },
            );
        }
        //     }
        // }

        println!("time: {:?}", start.elapsed());
        texture
            .update(
                None,
                unsafe { std::mem::transmute::<&[u32], &[u8]>(&pixels) },
                4 * W as usize,
            )
            .unwrap();
        canvas.copy(&texture, None, None).unwrap();
        // canvas
        //     .copy(
        //         &test_texture,
        //         None,
        //         Some(sdl2::rect::Rect::new(256, 256, 256, 256)),
        //     )
        //     .unwrap();
        canvas.present();
        std::thread::sleep(Duration::from_secs_f32(1.0 / 60.0))
    }
}
