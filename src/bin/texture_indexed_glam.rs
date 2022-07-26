#![feature(step_trait)]
use std::time::Instant;

use glam::IVec3;
use rasterize::{
    clip_polygon, clip_polygon_inplace, level,
    math::{self, prelude::*},
    palette::{self, Framebuffer},
    test_texture, texpoly, texpoly_vec,
};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

fn main() {
    const ZOOM: u32 = 4;

    const WINDOW_SCALE: u32 = ZOOM;
    const W: u32 = 320 * 4 / ZOOM;
    const H: u32 = 240 * 4 / ZOOM;

    // let blank = 0x10u8;
    // let mut pixels = [blank; (W * H) as usize];

    let (palette, mapping_table) = palette::read_colormap();

    let mut fb = Framebuffer::new(W, H, &palette);

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
        // .software()
        .present_vsync()
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

    // https://excalidraw.com/#json=4721006165884928,0MUG2eCYGmj706dqxZThow
    // let mut points = [
    //     Vec3::new(-10.0, -10.0, 20.0),
    //     Vec3::new(-10.0, 10.0, 20.0),
    //     Vec3::new(10.0, 10.0, 20.0),
    //     Vec3::new(10.0, -10.0, 20.0),
    //     Vec3::new(-10.0, -10.0, 5.0),
    //     Vec3::new(-10.0, 10.0, 5.0),
    //     Vec3::new(10.0, 10.0, 5.0),
    //     Vec3::new(10.0, -10.0, 5.0),
    // ];
    let bitmap_wall = palette::read_pcx("assets/wall01.pcx");
    let bitmap_floor = palette::read_pcx("assets/floor01.pcx");
    let bitmaps = [&bitmap_wall, &bitmap_floor];

    // #[rustfmt::skip]
    // let quads = vec![
    //     // back
    //     ((0, 0.0, 0.0), (1, 0.0, th), (2, tw, th), (3, tw, 0.0), &bitmap_wall),
    //     // left
    //     ((0, 0.0, 0.0), (4, tw, 0.0), (5, tw, th), (1, 0.0, th), &bitmap_wall),
    //     // right
    //     ((3, 0.0, 0.0), (2, 0.0, th), (6, tw, th), (7, tw, 0.0), &bitmap_wall),
    //     // top
    //     ((0, 0.0, 0.0), (3, 0.0, th), (7, tw, th), (4, tw, 0.0), &bitmap_wall),
    //     // bottom
    //     ((1, 0.0, 0.0), (5, tw, 0.0), (6, tw, th), (2, 0.0, th), &bitmap_floor),
    // ];

    let floor = [
        b"................",
        b"................",
        b"................",
        b"................",
        b"................",
        b".....1111111....",
        b".....1..........",
        b".....1..........",
        b".....1111.......",
        b"........1.......",
        b"........111.....",
        b"........1.......",
        b"........1.......",
        b"........1.......",
        b".11111111.......",
        b"................",
    ];
    let mut level = level::Blockmap::new();
    level.add(IVec3::ZERO, &floor);
    let (points, quads) = level.get_polygons();
    // let mut new_triangle = VecDeque::new();
    let mut click_index = 0;
    let bitmap = palette::quantize(&palette, &test_texture::create());

    let mut l = Vec3::new(0.0, 0.0, -13.0);

    let (perspective_project, perspective_unproject) = math::perspective(W as f32, H as f32, 100.0);
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

    let frustum = rasterize::make_frustum(
        &[
            Vec2::ZERO,
            Vec2::new(W as f32 - 1.0, 0.0),
            Vec2::new(W as f32 - 1.0, H as f32 - 1.0),
            Vec2::new(0.0, H as f32 - 1.0),
        ],
        &perspective_unproject,
    );
    println!("frustum: {:?}", frustum);
    let mut proj = false;
    'mainloop: loop {
        let mut event_pump = sdl_context.event_pump().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                // Event::MouseWheel { y, .. } => {
                //     let y = y as f32;
                //     for p in points[0..4].iter_mut() {
                //         p.z += y;
                //     }
                // }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    // Keycode::Z => r += std::f32::consts::PI / 180.0,
                    // Keycode::X => r -= std::f32::consts::PI / 180.0,
                    // Keycode::A => l.x -= 1.0,
                    // Keycode::D => l.x += 1.0,
                    // Keycode::W => l.z += 1.0,
                    // Keycode::S => l.z -= 1.0,
                    // Keycode::D => debug_overdraw = !debug_overdraw,
                    Keycode::P => proj = !proj,
                    Keycode::T => draw_texels = !draw_texels,
                    Keycode::B => bayer_dither = !bayer_dither,
                    _ => (),
                },
                _ => {}
            }
        }

        let keyboard_state = event_pump.keyboard_state();

        let rot = glam::Mat3::from_rotation_y(-r);
        let forward = rot * Vec3::new(0.0, 0.0, -0.5);
        let right = rot * Vec3::new(0.5, 0.0, 0.0);
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::A) {
            l -= right;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::D) {
            l += right;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::W) {
            l += forward;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::S) {
            l -= forward;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Z) {
            r += std::f32::consts::PI / 180.0 / 1.0;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::X) {
            r -= std::f32::consts::PI / 180.0 / 1.0;
        }

        let camera_rot = glam::Mat3::from_rotation_y(r);
        // r += std::f32::consts::PI / 180.0;

        let mut color = 0x3b0103a5u32;
        let duplicate = 0xffaa55u32;
        // fb.framebuffer.fill(0x0u8);
        fb.clear();

        // for y in 0..240 {
        //     fb.framebuffer[(16 + y * W) as usize] = y as u8;
        // }

        let start = Instant::now();
        rasterize::rasterize::G_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let mut num_texel = 0;
        for (p0, p1, p2, p3, bi) in quads.iter().cloned() {
            color = (color << 1) | (color >> (32 - 1));
            let poly_indexed = [p0, p1, p2, p3];
            let mut poly = poly_indexed
                .iter()
                .map(|(i, u, v)| (camera_rot * (points[*i as usize] - l), Vec2::new(*u, *v)))
                .collect::<Vec<_>>();

            for p in frustum.iter() {
                // poly = clip_polygon(p.clone(), &poly[..]);
                clip_polygon_inplace(*p, &mut poly);
            }

            if poly.len() < 3 {
                continue;
            }
            let project_mat = glam::Mat4::perspective_rh(
                100.0 * std::f32::consts::PI / 180.0,
                320.0 / 240.0,
                0.1,
                1000.0,
            );

            // let project_mat = glam::Mat4::orthographic_rh(0.0, 1.0, 0.0, 1.0, 0.1, 1000.0);

            println!("proj: {:?}", project_mat);
            let poly = poly
                .iter()
                .map(|(p, t)| {
                    let vp = project_mat.project_point3(*p);
                    let v = perspective_project(*p);
                    println!("{:?} {:?} {:?}", p, vp, v);

                    if proj {
                        (vp.x * 160.0 + 160.0, vp.y * 120.0 + 120.0, -p.z, t.x, t.y)
                    } else {
                        (v.x, v.y, p.z, t.x, t.y)
                    }
                })
                .collect::<Vec<_>>();

            // let transform = |p| p;
            let colors = [
                0xff, 0xff00, 0xff0000, 0xffff, 0xff00ff, 0xffff00, 0xff8080, 0x80ff80, 0x8080ff,
                0x808080, 0x80, 0x8000, 0x800000,
            ];
            // let clipped_polygon = vec![transform(p0), transform(p1), transform(p2), transform(p3)]
            texpoly::draw_polygon(&poly[..], |x, y, z, u, v, aux| {
                if !(x >= 0 && x < W as i32 && y >= 0 && y < H as i32) {
                    return;
                }

                debug_assert!(x >= 0 && x < W as i32 && y >= 0 && y < H as i32);
                let x = x as usize;
                let y = y as usize;
                let pixel_index = y * W as usize + x;

                if !true {
                    // let pixel = unsafe { pixels.get_unchecked_mut(pixel_index) };
                    // if *pixel != blank && debug_overdraw {
                    //     *pixel = duplicate;
                    // } else {
                    //     if draw_texels {
                    //         let (ui, vi) = if bayer_dither {
                    //             (
                    //                 (u + BAYER4X4_F[y % 4][x % 4]) as usize,
                    //                 (v + BAYER4X4_F[y % 4][x % 4]) as usize,
                    //             )
                    //         } else {
                    //             (u as usize, v as usize)
                    //         };
                    //         let color = unsafe {
                    //             bitmap.get_unchecked(
                    //                 (vi % test_texture::TH) * test_texture::TW
                    //                     + (ui % test_texture::TW),
                    //             )
                    //         };
                    //         *pixel = color & 0xffffff;
                    //     } else {
                    //         *pixel = colors[aux as usize % colors.len()];
                    //     }
                    // }
                } else {
                    if z > fb.zbuffer[pixel_index] {
                        return;
                    }
                    fb.zbuffer[pixel_index] = z;
                    let zi = (31 + (z as usize / 2).saturating_sub(16)).min(47);
                    let u = u as usize % test_texture::TW;
                    let v = v as usize % test_texture::TH;
                    let texel_index = u + v * test_texture::TW;

                    fb.framebuffer[pixel_index] =
                        mapping_table[zi][bitmaps[bi][texel_index as usize] as usize];
                    // debug_assert!(zi < palette::NUM_GAMMA_RAMP);
                    // debug_assert!(pixel_index < fb.framebuffer.len());
                    // debug_assert!(texel_index < bitmap.len());
                    // unsafe {
                    //     *fb.framebuffer.get_unchecked_mut(pixel_index) =
                    //         *mapping_table.get_unchecked(zi).get_unchecked(
                    //             *bitmaps.get_unchecked(bi).get_unchecked(texel_index) as usize,
                    //         );
                    // };
                }
                num_texel += 1;
            });
        }

        let dt = start.elapsed();
        println!(
            "time: {:?} {} {} MTx/s {}",
            dt,
            num_texel,
            num_texel as f32 * 1e-6 / dt.as_secs_f32(),
            (dt.as_secs_f32() / num_texel as f32) * 2e9
        );
        // texture
        //     .update(
        //         None,
        //         unsafe { std::mem::transmute::<&[u32], &[u8]>(&pixels) },
        //         4 * W as usize,
        //     )
        //     .unwrap();
        fb.upload(&mut texture);
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
}
