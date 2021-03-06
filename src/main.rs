#![feature(step_trait)]
use std::{collections::VecDeque, fmt::Debug};

use rasterize::{
    rasterize::{rasterize_triangle, Slope},
    slope::SlopeData,
};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

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
            let result = [
                SlopeData::new(from.0, to.0, num_steps),
                SlopeData::new(from.2, to.2, num_steps),
                SlopeData::new(from.3, to.3, num_steps),
                SlopeData::new(from.4, to.4, num_steps),
            ];
            result
        },
        //scanline function
        |y, left, right, _| {
            let xstart = left[0].get() as i32;
            let xend = right[0].get() as i32;

            let num_steps = xend - xstart;
            let mut props = [
                SlopeData::new(left[1].get(), right[1].get(), num_steps as f32),
                SlopeData::new(left[2].get(), right[2].get(), num_steps as f32),
                SlopeData::new(left[3].get(), right[3].get(), num_steps as f32),
            ];
            for x in xstart..xend {
                fragment(x as f32, y, props[0].get(), props[1].get(), props[2].get());
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
    const W: u32 = 320;
    const H: u32 = 240;
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

    let mut triangles = vec![
        ([10.0, 10.0], [20.0, 100.0], [90.0, 50.0]),
        // ([20, 10], [20, 100], [90, 50]),
    ];
    let mut new_triangle = VecDeque::new();
    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::MouseButtonDown { x, y, .. } => {
                    new_triangle.push_back([x as f32, y as f32]);
                    if new_triangle.len() == 3 {
                        triangles.push((new_triangle[0], new_triangle[1], new_triangle[2]));
                        new_triangle.pop_front();
                        // new_triangle.clear();
                    }
                }
                _ => {}
            }
        }

        let mut color = 0x3b0103a5u32;
        let blank = 0xffffffu32;
        let duplicate = 0xffaa55u32;
        let mut pixels = [blank; (W * H) as usize];

        for (p0, p1, p2) in triangles.iter().cloned() {
            color = (color << 1) | (color >> (32 - 1));

            let p0 = (p0[0], p0[1], 255.0, 0.0, 0.0);
            let p1 = (p1[0], p1[1], 0.0, 255.0, 0.0);
            let p2 = (p2[0], p2[1], 0.0, 0.0, 255.0);

            draw_polygon(p0, p1, p2, |x, y, r, g, b| {
                let x = x as u32;
                let y = y as u32;
                let pixel = &mut pixels[(y * W + x) as usize];
                if *pixel != blank {
                    *pixel = duplicate;
                } else {
                    let color = (r as u32) << 16 | (g as u32) << 8 | b as u32;
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
        canvas.present();
    }
}
