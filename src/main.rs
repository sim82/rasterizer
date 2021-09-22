#![feature(step_trait)]
use std::{fmt::Debug, fs::File, io::BufWriter, iter::Step, path::Path};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Point,
};
// pub trait Number:
//     PartialOrd + Copy + Sub<Output = Self> + Mul<Output = Self> + Into<f32> + Into<isize> + Step
// {
// }

pub trait Point2d<T> {
    fn get_xy(&self) -> (T, T);
    fn get_x(&self) -> T;
    fn get_y(&self) -> T;
}
#[derive(Debug)]
pub struct SlopeData {
    begin: f32,
    step: f32,
}

pub fn rasterize_triangle<T, P, G, S, M, D>(
    mut p0: P,
    mut p1: P,
    mut p2: P,
    get_xy: G,
    make_slope: M,
    mut draw_scanline: D,
) where
    T: num_traits::PrimInt + Step,
    G: Fn(&P) -> (T, T),
    S: Debug,
    M: Fn(&P, &P, T) -> S,
    D: FnMut(T, &mut [&mut S; 2]),
{
    let (mut x0, mut y0) = get_xy(&p0);
    let (mut x1, mut y1) = get_xy(&p1);
    let (mut x2, mut y2) = get_xy(&p2);

    if (y1, x1) < (y0, x0) {
        std::mem::swap(&mut p1, &mut p0);
        std::mem::swap(&mut x1, &mut x0);
        std::mem::swap(&mut y1, &mut y0);
    }
    if (y2, x2) < (y0, x0) {
        std::mem::swap(&mut p2, &mut p0);
        std::mem::swap(&mut x2, &mut x0);
        std::mem::swap(&mut y2, &mut y0);
    }
    if (y2, x2) < (y1, x1) {
        std::mem::swap(&mut p2, &mut p1);
        std::mem::swap(&mut x2, &mut x1);
        std::mem::swap(&mut y2, &mut y1);
    }

    if y0 == y2 {
        return;
    }
    let shortside_right = (y1 - y0) * (x2 - x0) < (x1 - x0) * (y2 - y0);
    let mut long_side = make_slope(&p0, &p2, y2 - y0);

    if y0 < y1 {
        let mut short_side = make_slope(&p0, &p1, y1 - y0);
        let mut sides = if shortside_right {
            [&mut long_side, &mut short_side]
        } else {
            [&mut short_side, &mut long_side]
        };
        // println!("1: {:?} {:?}", sides[0], sides[1]);
        for y in y0..y1 {
            draw_scanline(y, &mut sides);
        }
    }
    if y1 < y2 {
        let mut short_side = make_slope(&p1, &p2, y2 - y1);

        let mut sides = if shortside_right {
            [&mut long_side, &mut short_side]
        } else {
            [&mut short_side, &mut long_side]
        };
        // println!("2: {:?} {:?}", sides[0], sides[1]);

        for y in y1..y2 {
            draw_scanline(y, &mut sides);
        }
    }
}
fn draw_polygon<P>(p0: [i32; 2], p1: [i32; 2], p2: [i32; 2], mut plot: P)
where
    P: FnMut(i32, i32),
{
    rasterize_triangle(
        p0,
        p1,
        p2,
        |p| (p[0], p[1]),
        |from, to, num_steps| {
            let begin = from[0] as f32;
            let end = to[0] as f32;
            let inv_step = 1.0 / num_steps as f32;
            SlopeData {
                begin,
                step: (end - begin) * inv_step,
            }
        },
        |y, borders| {
            let xstart = borders[0].begin as i32;
            let xend = borders[1].begin as i32;

            for x in xstart..xend {
                plot(x, y);
            }
            borders[0].begin += borders[0].step;
            borders[1].begin += borders[1].step;
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
        ([10, 10], [20, 100], [90, 50]),
        ([20, 10], [20, 100], [90, 50]),
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
                    new_triangle.push_back([x, y]);
                    if new_triangle.len() == 3 {
                        triangles.push((new_triangle[0], new_triangle[1], new_triangle[2]));
                        new_triangle.pop_front();
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
            draw_polygon(p0, p1, p2, |x, y| {
                let x = x as u32;
                let y = y as u32;
                let pixel = &mut pixels[(y * W + x) as usize];
                if *pixel != blank {
                    *pixel = duplicate;
                } else {
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
