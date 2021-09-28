#![feature(step_trait)]
use std::{collections::VecDeque, fmt::Debug, time::Duration};

use num_traits::pow;
use rasterize::{
    math::{self, prelude::*},
    rasterize::{rasterize_triangle, Slope},
    test_texture,
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseWheelDirection, pixels::PixelFormatEnum};

#[derive(Debug)]
pub struct SlopeData {
    begin: f32,
    step: f32,
}
impl SlopeData {
    fn new(begin: f32, end: f32, num_steps: f32) -> SlopeData {
        let inv_step = 1.0 / num_steps as f32;
        SlopeData {
            begin: begin as f32,
            step: (end - begin) as f32 * inv_step,
        }
    }
}
impl Slope for SlopeData {
    fn get(&self) -> f32 {
        self.begin
    }

    fn advance(&mut self) {
        self.begin += self.step;
    }
}

// type Point = [i32; 5];
type Point = (f32, f32, f32, f32, f32);

fn draw_polygon<F>(p0: Point, p1: Point, p2: Point, mut fragment: F)
where
    F: FnMut(f32, f32, f32, f32, f32),
{
    rasterize_triangle(
        p0,
        p1,
        p2,
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
        |y, left, right| {
            let xstart = left[0].get();
            let xend = right[0].get();

            let num_steps = xend - xstart;
            let mut props = [
                SlopeData::new(left[1].get(), right[1].get(), num_steps),
                SlopeData::new(left[2].get(), right[2].get(), num_steps),
                SlopeData::new(left[3].get(), right[3].get(), num_steps),
                // SlopeData::new(left[3].get() as i32, right[3].get() as i32, num_steps),
            ];
            for x in (xstart as i32)..(xend as i32) {
                let z = 1.0 / props[0].get();
                fragment(
                    x as f32,
                    y,
                    z,
                    props[1].get() * z,
                    props[2].get() * z,
                    // props[2].get() as i32,
                );
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

    let mut triangles = vec![
        (
            [xrect[0], yrect[0], zrect[0], 0.0, 0.0],
            [xrect[1], yrect[1], zrect[1], 0.0, th],
            [xrect[2], yrect[2], zrect[2], tw, th],
        ),
        (
            [xrect[2], yrect[2], zrect[2], tw, th],
            [xrect[3], yrect[3], zrect[3], tw, 0.0],
            [xrect[0], yrect[0], zrect[0], 0.0, 0.0],
        ),
        // ([20, 10], [20, 100], [90, 50]),
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

    let (perspective_project, perspective_unproject) = math::perspective(W as f32, H as f32, 90.0);

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
                    triangles[0].0[2] += y;
                    triangles[0].1[2] += y;
                    triangles[1].2[2] += y;
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

                    triangles = vec![
                        (
                            [xrect[0], yrect[0], zrect[0], 0.0, 0.0],
                            [xrect[1], yrect[1], zrect[1], 0.0, th],
                            [xrect[2], yrect[2], zrect[2], tw, th],
                        ),
                        (
                            [xrect[2], yrect[2], zrect[2], tw, th],
                            [xrect[3], yrect[3], zrect[3], tw, 0.0],
                            [xrect[0], yrect[0], zrect[0], 0.0, 0.0],
                        ),
                        // ([20, 10], [20, 100], [90, 50]),
                    ];
                    click_index += 1;
                }
                _ => {}
            }
        }

        let mut color = 0x3b0103a5u32;
        let duplicate = 0xffaa55u32;
        pixels.fill(blank);
        for (p0, p1, p2) in triangles.iter().cloned() {
            color = (color << 1) | (color >> (32 - 1));

            let p0 = (p0[0], p0[1], p0[2], p0[3], p0[4]);
            let p1 = (p1[0], p1[1], p1[2], p1[3], p1[4]);
            let p2 = (p2[0], p2[1], p2[2], p2[3], p2[4]);

            let transform = |(x, y, z, u, v)| {
                let p3 = Vec3::new(x, y, z) - l;
                let (vx, vy) = perspective_project(p3).into();
                (vx, vy, p3.z, u, v)
            };

            // let transform = |p| p;

            draw_polygon(
                transform(p0),
                transform(p1),
                transform(p2),
                |x, y, z, u, v| {
                    let x = x as u32;
                    let y = y as u32;
                    let pixel_index = (y * W + x) as usize;
                    let mut bad_pixel = 0;
                    let pixel = if pixel_index < pixels.len() {
                        unsafe { pixels.get_unchecked_mut(pixel_index) }
                    } else {
                        &mut bad_pixel
                    };
                    // if *pixel != blank {
                    //     *pixel = duplicate;
                    // } else {
                    let color = bitmap[(v as usize % test_texture::TH) * test_texture::TW
                        + (u as usize % test_texture::TW)];
                    *pixel = color & 0xffffff;
                    // }
                },
            );
        }

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
