#![feature(step_trait)]
use std::time::Instant;

use rasterize::{
    math::{self, prelude::*},
    test_texture, texpoly, texpoly_vec,
};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

fn main() {
    const ZOOM: u32 = 1;

    const WINDOW_SCALE: u32 = ZOOM;
    const W: u32 = 424 * 4 / ZOOM;
    const H: u32 = 240 * 4 / ZOOM;

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
    let quads = vec![
        // back
        ((0, 0.0, 0.0), (1, 0.0, th), (2, tw, th), (3, tw, 0.0)),
        // left
        ((0, 0.0, 0.0), (4, tw, 0.0), (5, tw, th), (1, 0.0, th)),
        // right
        ((3, 0.0, 0.0), (2, 0.0, th), (6, tw, th), (7, tw, 0.0)),
        // top
        ((0, 0.0, 0.0), (3, 0.0, th), (7, tw, th), (4, tw, 0.0)),
        // bottom
        ((1, 0.0, 0.0), (5, tw, 0.0), (6, tw, th), (2, 0.0, th)),
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

    let l = Vec3::new(0.0, 0.0, -13.0);

    let (perspective_project, perspective_unproject) = math::perspective(W as f32, H as f32, 90.0);
    let mut r = 0.0;
    let mut debug_overdraw = false;
    let mut draw_texels = true;
    let mut bayer_dither = false;
    const BAYER4X4_F: [[f32; 4]; 4] = [
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
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::A => r += std::f32::consts::PI / 180.0,
                    Keycode::Z => r -= std::f32::consts::PI / 180.0,
                    Keycode::D => debug_overdraw = !debug_overdraw,
                    Keycode::T => draw_texels = !draw_texels,
                    Keycode::B => bayer_dither = !bayer_dither,
                    _ => (),
                },
                Event::MouseButtonDown { x, y, .. } => {
                    // xrect[click_index % 4] = x as f32;
                    // yrect[click_index % 4] = y as f32;
                    let point = (IVec2::new(x, y) / WINDOW_SCALE as i32).as_vec2();

                    let z = if (click_index % 4) < 2 { 20.0 } else { 5.0 };
                    let z = z - l.z;
                    let (wx, wy, _wz) = (perspective_unproject(point, z) + l).into();
                    xrect[click_index % 4] = wx;
                    yrect[click_index % 4] = wy;

                    click_index += 1;
                }
                _ => {}
            }
        }

        let camera_rot = glam::Mat3::from_rotation_z(r);
        // r += std::f32::consts::PI / 180.0;

        let mut color = 0x3b0103a5u32;
        let duplicate = 0xffaa55u32;
        pixels.fill(blank);

        let start = Instant::now();
        rasterize::rasterize::G_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let mut num_texel = 0;
        for (p0, p1, p2, p3) in quads.iter().cloned() {
            color = (color << 1) | (color >> (32 - 1));

            let transform = |(index, u, v)| {
                let p3 = camera_rot * (points[index] - l);
                let (vx, vy) = perspective_project(p3).into();
                (vx, vy, p3.z, u, v)
            };

            // let transform = |p| p;
            let colors = [
                0xff, 0xff00, 0xff0000, 0xffff, 0xff00ff, 0xffff00, 0xff8080, 0x80ff80, 0x8080ff,
                0x808080, 0x80, 0x8000, 0x800000,
            ];
            texpoly_vec::draw_polygon(
                &[transform(p0), transform(p1), transform(p2), transform(p3)],
                // |x, y, _z, u, v, aux| {
                //     if x < 0 || x >= W as i32 || y < 0 || y >= H as i32 {
                //         return;
                //     }
                //     let x = x as usize;
                //     let y = y as usize;
                //     let pixel_index = y * W as usize + x;
                //     let pixel = unsafe { pixels.get_unchecked_mut(pixel_index) };
                //     if *pixel != blank && debug_overdraw {
                //         *pixel = duplicate;
                //     } else {
                //         // let ui = (u + bayer4x4_f[y % 4][x % 4]) as usize;
                //         // let vi = (v + bayer4x4_f[y % 4][x % 4]) as usize;
                //         if draw_texels {
                //             let (ui, vi) = if bayer_dither {
                //                 (
                //                     (u + BAYER4X4_F[y % 4][x % 4]) as usize,
                //                     (v + BAYER4X4_F[y % 4][x % 4]) as usize,
                //                 )
                //             } else {
                //                 (u as usize, v as usize)
                //             };
                //             let color = unsafe {
                //                 bitmap.get_unchecked(
                //                     (vi % test_texture::TH) * test_texture::TW
                //                         + (ui % test_texture::TW),
                //                 )
                //             };
                //             *pixel = color & 0xffffff;
                //         } else {
                //             *pixel = colors[aux as usize % colors.len()];
                //         }
                //     }
                //     num_texel += 1;
                // },
                |x, y, _z, u, v, aux| {
                    if x < 0 || x >= W as i32 || y < 0 || y >= H as i32 {
                        return;
                    }
                    let x = x as usize;
                    let y = y as usize;
                    let pixel_index = y * W as usize + x;

                    let u = u as usize % test_texture::TW;
                    let v = v as usize % test_texture::TH;
                    let texel_index = u + v * test_texture::TW;

                    // let pixel = unsafe { pixels.get_unchecked_mut(pixel_index) };
                    unsafe {
                        *pixels.get_unchecked_mut(pixel_index) = *bitmap.get_unchecked(texel_index)
                    };

                    num_texel += 1;
                },
            );
        }

        let dt = start.elapsed();
        println!(
            "time: {:?} {} {} MTx/s {}",
            dt,
            num_texel,
            num_texel as f32 * 1e-6 / dt.as_secs_f32(),
            (dt.as_secs_f32() / num_texel as f32) * 2e9
        );
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
        // std::thread::sleep(Duration::from_secs_f32(1.0 / 60.0))
    }
}
