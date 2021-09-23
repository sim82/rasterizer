#![feature(step_trait)]
use std::{collections::VecDeque, fmt::Debug};

use num_traits::pow;
use rasterize::{
    rasterize::{rasterize_triangle, Slope},
    test_texture,
};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

#[derive(Debug)]
pub struct SlopeData {
    begin: f32,
    step: f32,
}
impl SlopeData {
    fn new(begin: i32, end: i32, num_steps: i32) -> SlopeData {
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
type Point = (i32, i32, i32, i32);

fn draw_polygon<F>(p0: Point, p1: Point, p2: Point, mut fragment: F)
where
    F: FnMut(i32, i32, i32, i32),
{
    rasterize_triangle(
        p0,
        p1,
        p2,
        |p| (p.0, p.1),
        // slope generator
        |from, to, num_steps| {
            let result = [
                SlopeData::new(from.0, to.0, num_steps),
                SlopeData::new(from.2, to.2, num_steps),
                SlopeData::new(from.3, to.3, num_steps),
                // SlopeData::new(from.4, to.4, num_steps),
            ];
            result
        },
        //scanline function
        |y, left, right| {
            let xstart = left[0].get() as i32;
            let xend = right[0].get() as i32;

            let num_steps = xend - xstart;
            let mut props = [
                SlopeData::new(left[1].get() as i32, right[1].get() as i32, num_steps),
                SlopeData::new(left[2].get() as i32, right[2].get() as i32, num_steps),
                // SlopeData::new(left[3].get() as i32, right[3].get() as i32, num_steps),
            ];
            for x in xstart..xend {
                fragment(
                    x,
                    y,
                    props[0].get() as i32,
                    props[1].get() as i32,
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
    const W: u32 = 800;
    const H: u32 = 600;
    let mut pixels = [20u8; (W * H * 4) as usize];

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", W, H)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .software()
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

    let tw = test_texture::TW as i32;
    let th = test_texture::TH as i32;

    // let mut x0 = 10;
    // let mut y0 = 10;
    // let mut x1 = 10;
    // let mut y1 = 100;
    // let mut x2 = 100;
    // let mut y2 = 100;
    // let mut x3 = 100;
    // let mut y3 = 10;

    let mut xrect = [10, 10, 100, 100];
    let mut yrect = [10, 100, 100, 10];

    let mut triangles = vec![
        (
            [xrect[0], yrect[0], 0, 0],
            [xrect[1], yrect[1], 0, th],
            [xrect[2], yrect[2], tw, th],
        ),
        (
            [xrect[2], yrect[2], tw, th],
            [xrect[3], yrect[3], tw, 0],
            [xrect[0], yrect[0], 0, 0],
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

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::MouseButtonDown { x, y, .. } => {
                    xrect[click_index % 4] = x;
                    yrect[click_index % 4] = y;

                    triangles = vec![
                        (
                            [xrect[0], yrect[0], 0, 0],
                            [xrect[1], yrect[1], 0, th],
                            [xrect[2], yrect[2], tw, th],
                        ),
                        (
                            [xrect[2], yrect[2], tw, th],
                            [xrect[3], yrect[3], tw, 0],
                            [xrect[0], yrect[0], 0, 0],
                        ),
                        // ([20, 10], [20, 100], [90, 50]),
                    ];
                    click_index += 1;
                }
                _ => {}
            }
        }

        let mut color = 0x3b0103a5u32;
        let blank = 0x0u32;
        let duplicate = 0xffaa55u32;
        let mut pixels = [blank; (W * H) as usize];

        for (p0, p1, p2) in triangles.iter().cloned() {
            color = (color << 1) | (color >> (32 - 1));

            let p0 = (p0[0], p0[1], p0[2], p0[3]);
            let p1 = (p1[0], p1[1], p1[2], p1[3]);
            let p2 = (p2[0], p2[1], p2[2], p2[3]);

            draw_polygon(p0, p1, p2, |x, y, u, v| {
                let x = x as u32;
                let y = y as u32;
                let pixel = &mut pixels[(y * W + x) as usize];
                if *pixel != blank {
                    *pixel = duplicate;
                } else {
                    let color = bitmap[(u as usize % test_texture::TH) * test_texture::TW
                        + (v as usize % test_texture::TW)];
                    *pixel = color & 0xffffff;
                }
            });
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
    }
}
